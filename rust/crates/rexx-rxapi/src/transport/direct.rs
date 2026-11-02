//! A `Core` called in process, for test isolation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use super::{CancelToken, Transport, TransportError};
use crate::core::{ClientId, Core, Outcome};
use crate::message::{Reply, Request};

struct State {
    core: Core,
    next_connection: u64,
    replies: HashMap<ClientId, Reply>,
}

struct Hub {
    state: Mutex<State>,
    delivered: Condvar,
}

impl Hub {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("no client panicked in the core")
    }
}

/// One client of an in-process core. Dropping it closes the client.
pub struct Direct {
    hub: Arc<Hub>,
    connection: u64,
    next_call: AtomicU64,
}

impl Direct {
    /// The first client of a fresh core.
    pub fn new() -> Direct {
        let state = State {
            core: Core::new(),
            next_connection: 1,
            replies: HashMap::new(),
        };
        Direct {
            hub: Arc::new(Hub {
                state: Mutex::new(state),
                delivered: Condvar::new(),
            }),
            connection: 0,
            next_call: AtomicU64::new(0),
        }
    }

    /// Another client of the same core.
    pub fn client(&self) -> Direct {
        let mut state = self.hub.state();
        let connection = state.next_connection;
        state.next_connection += 1;
        Direct {
            hub: Arc::clone(&self.hub),
            connection,
            next_call: AtomicU64::new(0),
        }
    }
}

impl Default for Direct {
    fn default() -> Direct {
        Direct::new()
    }
}

impl Transport for Direct {
    fn call_cancellable(
        &self,
        request: Request,
        token: &CancelToken,
    ) -> Result<Reply, TransportError> {
        let mut started = token.started();
        if started.cancelled {
            return Ok(Reply::Cancelled);
        }
        let id = ClientId::connection(self.connection)
            .call(self.next_call.fetch_add(1, Ordering::Relaxed));
        started.call = Some(id.call);
        let mut state = self.hub.state();
        let outcome = state.core.handle(id, request);
        drop(started);
        let ready = state.core.take_ready();
        if !ready.is_empty() {
            state.replies.extend(ready);
            self.hub.delivered.notify_all();
        }
        if let Outcome::Reply(reply) = outcome {
            return Ok(reply);
        }
        loop {
            if let Some(reply) = state.replies.remove(&id) {
                return Ok(reply);
            }
            state = self
                .hub
                .delivered
                .wait(state)
                .expect("no client panicked in the core");
        }
    }
}

impl Drop for Direct {
    fn drop(&mut self) {
        self.hub
            .state()
            .core
            .client_closed(ClientId::connection(self.connection));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::code::{OK, QUEUE_ACCESS, QUEUE_NOT_REGISTERED};
    use crate::message::{FIFO, Item, Timestamp, WAIT};

    fn open(client: &Direct) {
        client
            .call(Request::OpenQueue {
                name: Some(b"q".to_vec()),
            })
            .expect("direct calls do not fail");
    }

    fn delete(client: &Direct) -> Reply {
        client
            .call(Request::DeleteQueue {
                name: Some(b"q".to_vec()),
            })
            .expect("direct calls do not fail")
    }

    fn pull() -> Request {
        Request::PullQueue {
            session: 1,
            name: Some(b"q".to_vec()),
            wait: WAIT,
        }
    }

    #[test]
    fn a_blocked_pull_returns_when_another_client_adds() {
        let client = Direct::new();
        open(&client);
        let puller = client.client();
        let thread = std::thread::spawn(move || {
            loop {
                open(&puller);
                let reply = puller.call(pull());
                let gap = Reply::Pulled {
                    status: QUEUE_NOT_REGISTERED,
                    item: None,
                };
                if reply != Ok(gap) {
                    return reply;
                }
            }
        });
        while delete(&client) != Reply::Status(QUEUE_ACCESS) {
            std::thread::yield_now();
            open(&client);
        }

        let add = Request::AddQueue {
            session: 1,
            name: Some(b"q".to_vec()),
            data: b"item".to_vec(),
            flag: FIFO,
            time: Timestamp::default(),
        };
        assert_eq!(client.call(add), Ok(Reply::Status(OK)));

        assert_eq!(
            thread.join().expect("the puller"),
            Ok(Reply::Pulled {
                status: OK,
                item: Some(Item {
                    data: b"item".to_vec(),
                    time: Timestamp::default(),
                }),
            })
        );
    }

    #[test]
    fn a_cancelled_pull_returns_and_the_client_stays_usable() {
        let client = Arc::new(Direct::new());
        open(&client);
        let token = CancelToken::default();
        let blocked = {
            let (client, token) = (Arc::clone(&client), token.clone());
            std::thread::spawn(move || {
                loop {
                    open(&client);
                    let reply = client.call_cancellable(pull(), &token);
                    let gap = Reply::Pulled {
                        status: QUEUE_NOT_REGISTERED,
                        item: None,
                    };
                    if reply != Ok(gap) {
                        return reply;
                    }
                }
            })
        };
        while delete(&client) != Reply::Status(QUEUE_ACCESS) {
            std::thread::yield_now();
            open(&client);
        }

        client.cancel(&token).expect("direct calls do not fail");

        assert_eq!(blocked.join().expect("the puller"), Ok(Reply::Cancelled));
        assert_eq!(delete(&client), Reply::Status(OK));
    }

    #[test]
    fn dropping_a_client_closes_it_for_the_core() {
        let client = Direct::new();
        let session = client.client();
        let query = |client: &Direct| {
            client
                .call(Request::QueryQueue {
                    session: 9,
                    name: None,
                })
                .expect("direct calls do not fail")
        };
        session
            .call(Request::AddQueue {
                session: 9,
                name: None,
                data: Vec::new(),
                flag: FIFO,
                time: Timestamp::default(),
            })
            .expect("direct calls do not fail");
        drop(session);
        assert_eq!(
            query(&client),
            Reply::QueueCount {
                status: OK,
                count: Some(0)
            }
        );
    }
}
