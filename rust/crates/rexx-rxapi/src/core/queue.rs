//! Named and session queues, and pulls waiting on them.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

use super::{ClientId, Outcome, fold};
use crate::message::code::*;
use crate::message::{FIFO, Item, LIFO, MAX_QUEUE_NAME, NO_WAIT, Reply, Timestamp, WAIT};

/// A queue. While `waiters` is non-empty `items` is empty.
#[derive(Default)]
struct Queue {
    items: VecDeque<Item>,
    waiters: VecDeque<ClientId>,
}

impl Queue {
    fn add(&mut self, item: Item, flag: u64, ready: &mut Vec<(ClientId, Reply)>) {
        if let Some(waiter) = self.waiters.pop_front() {
            let reply = Reply::Pulled {
                status: OK,
                item: Some(item),
            };
            ready.push((waiter, reply));
        } else if flag == LIFO {
            self.items.push_front(item);
        } else {
            self.items.push_back(item);
        }
    }

    fn pull(&mut self, client: ClientId, wait: u64) -> Outcome {
        match self.items.pop_front() {
            Some(item) => Outcome::Reply(Reply::Pulled {
                status: OK,
                item: Some(item),
            }),
            None if wait == WAIT => {
                self.waiters.push_back(client);
                Outcome::Parked
            }
            None => Outcome::Reply(Reply::Pulled {
                status: QUEUE_EMPTY,
                item: None,
            }),
        }
    }
}

/// A session queue and the connections using it.
#[derive(Default)]
struct Session {
    queue: Queue,
    users: HashSet<u64>,
}

/// Which queue an operation that treats NULL as the session queue addresses.
enum Target {
    Session,
    Named(Vec<u8>),
}

#[derive(Default)]
pub(super) struct Queues {
    /// By folded name. The empty key is the hidden queue a NULL name reaches.
    named: HashMap<Vec<u8>, Queue>,
    sessions: HashMap<u64, Session>,
    next_tag: u64,
    ready: Vec<(ClientId, Reply)>,
}

fn is_valid(name: &[u8]) -> bool {
    (1..=MAX_QUEUE_NAME).contains(&name.len())
        && name
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || b".?!_".contains(byte))
}

fn is_session(name: &[u8]) -> bool {
    name.eq_ignore_ascii_case(b"SESSION")
}

/// Resolves a name for the operations where NULL and `SESSION` both mean the session queue.
fn target(name: Option<Vec<u8>>) -> Result<Target, u64> {
    match name {
        None => Ok(Target::Session),
        Some(name) if !is_valid(&name) => Err(QUEUE_BAD_NAME),
        Some(name) if is_session(&name) => Ok(Target::Session),
        Some(name) => Ok(Target::Named(fold(&name))),
    }
}

impl Queues {
    fn session(&mut self, client: ClientId, handle: u64) -> &mut Queue {
        let session = self.sessions.entry(handle).or_default();
        session.users.insert(client.connection);
        &mut session.queue
    }

    /// The queue `target` names, if there is one.
    fn resolve(
        &mut self,
        client: ClientId,
        handle: u64,
        name: Option<Vec<u8>>,
    ) -> Result<&mut Queue, u64> {
        match target(name)? {
            Target::Session => Ok(self.session(client, handle)),
            Target::Named(key) => self.named.get_mut(&key).ok_or(QUEUE_NOT_REGISTERED),
        }
    }

    pub(super) fn attach(&mut self, client: ClientId, handle: u64) -> Reply {
        self.session(client, handle);
        Reply::Status(OK)
    }

    pub(super) fn create(&mut self, session: u64, requested: Option<Vec<u8>>, size: u64) -> Reply {
        let refused = |status| Reply::QueueCreated {
            status,
            name: None,
            duplicate: None,
        };
        let mut duplicate = false;
        if let Some(requested) = requested {
            if requested.len() as u64 >= size {
                return refused(QUEUE_STORAGE);
            }
            if !is_valid(&requested) || is_session(&requested) {
                return refused(QUEUE_BAD_NAME);
            }
            if let Entry::Vacant(vacant) = self.named.entry(fold(&requested)) {
                vacant.insert(Queue::default());
                return Reply::QueueCreated {
                    status: OK,
                    name: Some(requested),
                    duplicate: Some(false),
                };
            }
            duplicate = true;
        }
        let mut name = self.generated_name(session);
        self.named.insert(fold(&name), Queue::default());
        let capacity = size.saturating_sub(1) as usize;
        let status = if name.len() > capacity {
            name.truncate(capacity);
            QUEUE_STORAGE
        } else {
            OK
        };
        Reply::QueueCreated {
            status,
            name: Some(name),
            duplicate: Some(duplicate),
        }
    }

    /// A valid name no queue has, which never fits a 10-byte buffer.
    fn generated_name(&mut self, session: u64) -> Vec<u8> {
        loop {
            let name = format!("S{session:x}Q{:08x}", self.next_tag).into_bytes();
            self.next_tag += 1;
            if !self.named.contains_key(&fold(&name)) {
                return name;
            }
        }
    }

    pub(super) fn open(&mut self, name: Option<Vec<u8>>) -> Reply {
        let opened = |status, duplicate| Reply::QueueOpened { status, duplicate };
        match name {
            Some(name) if !is_valid(&name) => opened(QUEUE_BAD_NAME, None),
            Some(name) if is_session(&name) => opened(OK, Some(true)),
            name => {
                let key = fold(&name.unwrap_or_default());
                let existed = self.named.contains_key(&key);
                self.named.entry(key).or_default();
                opened(OK, Some(existed))
            }
        }
    }

    pub(super) fn exists(&self, name: Option<Vec<u8>>) -> Reply {
        Reply::Status(match name {
            Some(name) if !is_valid(&name) => QUEUE_BAD_NAME,
            Some(name) if is_session(&name) => OK,
            name => {
                if self.named.contains_key(&fold(&name.unwrap_or_default())) {
                    OK
                } else {
                    QUEUE_NOT_REGISTERED
                }
            }
        })
    }

    pub(super) fn delete(&mut self, name: Option<Vec<u8>>) -> Reply {
        if name
            .as_ref()
            .is_some_and(|name| !is_valid(name) || is_session(name))
        {
            return Reply::Status(QUEUE_BAD_NAME);
        }
        let key = fold(&name.unwrap_or_default());
        Reply::Status(match self.named.get(&key) {
            None => QUEUE_NOT_REGISTERED,
            Some(queue) if !queue.waiters.is_empty() => QUEUE_ACCESS,
            Some(_) => {
                self.named.remove(&key);
                OK
            }
        })
    }

    pub(super) fn clear(&mut self, client: ClientId, session: u64, name: Option<Vec<u8>>) -> Reply {
        Reply::Status(match self.resolve(client, session, name) {
            Ok(queue) => {
                queue.items.clear();
                OK
            }
            Err(status) => status,
        })
    }

    pub(super) fn query(&mut self, client: ClientId, session: u64, name: Option<Vec<u8>>) -> Reply {
        let (status, count) = match self.resolve(client, session, name) {
            Ok(queue) => (OK, Some(queue.items.len() as u64)),
            Err(QUEUE_NOT_REGISTERED) => (QUEUE_NOT_REGISTERED, Some(0)),
            Err(status) => (status, None),
        };
        Reply::QueueCount { status, count }
    }

    pub(super) fn add(
        &mut self,
        client: ClientId,
        session: u64,
        name: Option<Vec<u8>>,
        data: Vec<u8>,
        flag: u64,
        time: Timestamp,
    ) -> Reply {
        if flag != FIFO && flag != LIFO {
            return Reply::Status(QUEUE_PRIORITY);
        }
        let mut ready = std::mem::take(&mut self.ready);
        let status = match self.resolve(client, session, name) {
            Ok(queue) => {
                queue.add(Item { data, time }, flag, &mut ready);
                OK
            }
            Err(status) => status,
        };
        self.ready = ready;
        Reply::Status(status)
    }

    pub(super) fn pull(
        &mut self,
        client: ClientId,
        session: u64,
        name: Option<Vec<u8>>,
        wait: u64,
    ) -> Outcome {
        if wait != NO_WAIT && wait != WAIT {
            return Outcome::Reply(Reply::Pulled {
                status: QUEUE_BAD_WAIT_FLAG,
                item: None,
            });
        }
        match self.resolve(client, session, name) {
            Ok(queue) => queue.pull(client, wait),
            Err(status) => Outcome::Reply(Reply::Pulled { status, item: None }),
        }
    }

    pub(super) fn take_ready(&mut self) -> Vec<(ClientId, Reply)> {
        std::mem::take(&mut self.ready)
    }

    pub(super) fn connection_closed(&mut self, connection: u64) {
        let keeps = |waiter: &ClientId| waiter.connection != connection;
        for queue in self.named.values_mut() {
            queue.waiters.retain(keeps);
        }
        self.sessions.retain(|_, session| {
            session.queue.waiters.retain(keeps);
            session.users.remove(&connection);
            !session.users.is_empty()
        });
    }

    pub(super) fn no_named_queues(&self) -> bool {
        self.named.is_empty()
    }
}
