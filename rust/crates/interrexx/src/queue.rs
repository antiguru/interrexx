//! The external data queues: the session queue `PUSH` and `QUEUE` reach through `.STDQUE`,
//! held in process until a child could see it, and every named queue, which the API service
//! holds.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

use interrexx_rxapi::message::code::{OK, QUEUE_BAD_NAME, QUEUE_EMPTY};
use interrexx_rxapi::message::{FIFO, LIFO, NO_WAIT, Reply, Request, WAIT};
use interrexx_rxapi::transport::CancelToken;

use crate::dispatch::queue_methods::queue_failure;
use crate::rxapi_client;
use crate::{Failure, Interp};

/// A `lineIn` waiting for a line of `name`, with the service's pull where the queue is named.
pub(crate) struct QueueWait {
    name: Vec<u8>,
    remote: Option<RemoteWait>,
}

/// A waiting pull a helper thread holds in the service, and the slot its reply lands in.
#[derive(Default)]
struct RemoteWait {
    token: CancelToken,
    slot: Arc<(Mutex<Option<Result<Reply, u64>>>, Condvar)>,
}

impl RemoteWait {
    fn peek(&self) -> Option<()> {
        let (held, _) = &*self.slot;
        held.lock()
            .expect("the slot's critical section cannot panic")
            .as_ref()
            .map(drop)
    }

    /// The reply, waiting for it.
    fn take(&self) -> Result<Reply, u64> {
        let (held, filled) = &*self.slot;
        let mut held = held
            .lock()
            .expect("the slot's critical section cannot panic");
        loop {
            if let Some(reply) = held.take() {
                return reply;
            }
            held = filled
                .wait(held)
                .expect("the slot's critical section cannot panic");
        }
    }
}

/// Where the session queue lives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sharing {
    /// In process: no other process has its handle.
    Local,
    /// In the service, which this process's connection keeps it in.
    Remote,
}

/// The session queue, held in process until another process could see it.
///
/// `lines`' head is the next line a pull removes. Once remote, the session queue lives in the
/// service and `lines` stays empty.
pub(crate) struct Queue {
    lines: VecDeque<Vec<u8>>,
    sharing: Sharing,
}

/// The buffer size the interpreter's own creates pass, `MAX_QUEUE_NAME_LENGTH + 1`.
const NAME_BUFFER: u64 = 251;

impl Queue {
    /// A queue held in process, or one in the service where `inherited`.
    pub(crate) fn new(inherited: bool) -> Queue {
        Queue {
            lines: VecDeque::new(),
            sharing: match inherited {
                true => Sharing::Remote,
                false => Sharing::Local,
            },
        }
    }

    /// `PUSH line`: inserted at the head.
    pub(crate) fn push(&mut self, line: Vec<u8>) {
        self.lines.push_front(line);
    }

    /// `QUEUE line`: appended at the tail.
    pub(crate) fn queue(&mut self, line: Vec<u8>) {
        self.lines.push_back(line);
    }

    /// The head line, removed, or `None` when the queue is empty.
    pub(crate) fn pop(&mut self) -> Option<Vec<u8>> {
        self.lines.pop_front()
    }

    /// How many lines are waiting.
    pub(crate) fn len(&self) -> usize {
        self.lines.len()
    }
}

fn is_session(name: &[u8]) -> bool {
    name.eq_ignore_ascii_case(b"SESSION")
}

/// The status a reply carries.
fn status(reply: &Reply) -> u64 {
    match reply {
        Reply::Status(status)
        | Reply::QueueCreated { status, .. }
        | Reply::QueueOpened { status, .. }
        | Reply::QueueCount { status, .. }
        | Reply::Pulled { status, .. }
        | Reply::Registration { status, .. }
        | Reply::MacroPosition { status, .. }
        | Reply::MacroImage { status, .. }
        | Reply::Macros { status, .. } => *status,
        Reply::Cancelled => QUEUE_EMPTY,
    }
}

/// Appends `lines` to the service's session queue.
fn move_lines(client: &rxapi_client::ApiClient, lines: &mut VecDeque<Vec<u8>>) {
    for line in std::mem::take(lines) {
        let _ = client.call(Request::AddQueue {
            session: client.session(),
            name: None,
            data: line,
            flag: FIFO,
            time: Default::default(),
        });
    }
}

/// A waiting pull of `name` a helper thread holds in the service, or `None` without one.
fn remote_wait(name: &[u8]) -> Option<RemoteWait> {
    let client = rxapi_client::client().ok()?;
    let wait = RemoteWait::default();
    let request = Request::PullQueue {
        session: client.session(),
        name: Some(name.to_vec()),
        wait: WAIT,
    };
    let (token, slot) = (wait.token.clone(), Arc::clone(&wait.slot));
    std::thread::spawn(move || {
        crate::signal::block();
        let reply = client.call_cancellable(request, &token);
        let (held, filled) = &*slot;
        *held
            .lock()
            .expect("the slot's critical section cannot panic") = Some(reply);
        filled.notify_all();
    });
    Some(wait)
}

/// `request`'s status from the service.
fn call_status(request: Request) -> u64 {
    match rxapi_client::client().and_then(|client| client.call(request)) {
        Ok(reply) => status(&reply),
        Err(status) => status,
    }
}

impl Interp {
    /// Whether `name` reaches the session queue still held in process.
    pub(crate) fn queue_is_local(&self, name: &[u8]) -> bool {
        is_session(name) && self.queue.sharing == Sharing::Local
    }

    /// The session handle a request carries.
    fn session() -> u64 {
        rxapi_client::session_handle()
    }

    /// `RexxAddQueue(name, line, lifo)`.
    pub(crate) fn queue_add(&mut self, name: &[u8], line: Vec<u8>, lifo: bool) -> u64 {
        if self.queue_is_local(name) {
            match lifo {
                true => self.queue.push(line),
                false => self.queue.queue(line),
            }
            return OK;
        }
        call_status(Request::AddQueue {
            session: Self::session(),
            name: Some(name.to_vec()),
            data: line,
            flag: if lifo { LIFO } else { FIFO },
            time: Default::default(),
        })
    }

    /// `RexxPullFromQueue(name, NOWAIT)`: the head line, or the failure's code.
    pub(crate) fn queue_pull(&mut self, name: &[u8]) -> Result<Vec<u8>, u64> {
        if self.queue_is_local(name) {
            return self.queue.pop().ok_or(QUEUE_EMPTY);
        }
        let request = Request::PullQueue {
            session: Self::session(),
            name: Some(name.to_vec()),
            wait: NO_WAIT,
        };
        match rxapi_client::client()?.call(request)? {
            Reply::Pulled {
                item: Some(item), ..
            } => Ok(item.data),
            reply => Err(status(&reply)),
        }
    }

    /// `RexxQueryQueue(name)`'s count, 0 on any failure.
    pub(crate) fn queue_count(&self, name: &[u8]) -> u64 {
        if self.queue_is_local(name) {
            return self.queue.len() as u64;
        }
        let request = Request::QueryQueue {
            session: Self::session(),
            name: Some(name.to_vec()),
        };
        match rxapi_client::client().and_then(|client| client.call(request)) {
            Ok(Reply::QueueCount {
                count: Some(count), ..
            }) => count,
            _ => 0,
        }
    }

    /// `RexxClearQueue(name)`.
    pub(crate) fn queue_clear(&mut self, name: &[u8]) -> u64 {
        if self.queue_is_local(name) {
            self.queue.lines.clear();
            return OK;
        }
        call_status(Request::ClearQueue {
            session: Self::session(),
            name: Some(name.to_vec()),
        })
    }

    /// `RexxCreateQueue` into a buffer of `MAX_QUEUE_NAME_LENGTH + 1`: the name, or the code.
    pub(crate) fn queue_create(&self, requested: Option<&[u8]>) -> Result<Vec<u8>, u64> {
        if requested.is_some_and(is_session) {
            return Err(QUEUE_BAD_NAME);
        }
        let request = Request::CreateQueue {
            session: Self::session(),
            requested: requested.map(<[u8]>::to_vec),
            size: NAME_BUFFER,
        };
        match rxapi_client::client()?.call(request)? {
            Reply::QueueCreated {
                status: OK,
                name: Some(name),
                ..
            } => Ok(name),
            reply => Err(status(&reply)),
        }
    }

    /// `RexxOpenQueue(name)`.
    pub(crate) fn queue_open(&self, name: &[u8]) -> u64 {
        if is_session(name) {
            return OK;
        }
        call_status(Request::OpenQueue {
            name: Some(name.to_vec()),
        })
    }

    /// `RexxQueueExists(name)`.
    pub(crate) fn queue_exists(&self, name: &[u8]) -> u64 {
        if is_session(name) {
            return OK;
        }
        call_status(Request::QueueExists {
            name: Some(name.to_vec()),
        })
    }

    /// `RexxDeleteQueue(name)`.
    pub(crate) fn queue_delete(&self, name: &[u8]) -> u64 {
        if is_session(name) {
            return QUEUE_BAD_NAME;
        }
        call_status(Request::DeleteQueue {
            name: Some(name.to_vec()),
        })
    }

    /// Starts the running activity's wait for a line of `name`, which its park polls. A named
    /// queue's wait is a pull the service holds, so the queue counts as waited on.
    pub(crate) fn begin_queue_wait(&mut self, name: Vec<u8>) {
        let remote = match self.queue_is_local(&name) {
            true => None,
            false => remote_wait(&name),
        };
        self.activity.queue_wait = Some(Box::new(QueueWait { name, remote }));
    }

    /// Whether the running activity's wait is for the in-process session queue, which no other
    /// process can reach, while it is empty and no other activity is alive to add a line.
    pub(crate) fn queue_wait_is_unending(&self) -> bool {
        self.activity.queue_wait.as_ref().is_some_and(|wait| {
            wait.remote.is_none()
                && is_session(&wait.name)
                && self.queue.sharing == Sharing::Local
                && self.queue.lines.is_empty()
                && !self.others_live()
        })
    }

    /// Whether the running activity waits for a line that has not arrived.
    pub(crate) fn queue_wait_pending(&mut self) -> bool {
        let Some(mut wait) = self.activity.queue_wait.take() else {
            return false;
        };
        if wait.remote.is_none() && !self.queue_is_local(&wait.name) {
            // The session queue moved to the service during the wait: the wait goes on there.
            wait.remote = remote_wait(&wait.name);
        }
        let pending = match &wait.remote {
            Some(remote) => remote.peek().is_none(),
            None => self.queue.lines.is_empty(),
        };
        self.activity.queue_wait = Some(wait);
        pending
    }

    /// Ends the running activity's wait: its line, or `None` for a failure or, where `halted`,
    /// a wait the halt ended. A line that arrived as the halt did goes back to the queue's head.
    pub(crate) fn end_queue_wait(&mut self, halted: bool) -> Option<Vec<u8>> {
        let wait = self.activity.queue_wait.take()?;
        let Some(remote) = wait.remote else {
            if halted {
                return None;
            }
            return match self.queue_is_local(&wait.name) {
                true => self.queue.pop(),
                false => self.queue_pull(&wait.name).ok(),
            };
        };
        let client = rxapi_client::client().ok()?;
        if halted {
            client.cancel(&remote.token);
        }
        let line = match remote.take() {
            Ok(Reply::Pulled {
                item: Some(item), ..
            }) => item.data,
            _ => return None,
        };
        if !halted {
            return Some(line);
        }
        let _ = client.call(Request::AddQueue {
            session: client.session(),
            name: Some(wait.name),
            data: line,
            flag: LIFO,
            time: Default::default(),
        });
        None
    }

    /// Makes the session queue reachable from a child about to start: this process connects,
    /// which keeps the queue in the service while its connection lives, any line held here moves
    /// there, and the child inherits the handle. Without a service an empty queue stays here and
    /// the child gets no handle; a queue holding lines raises 48.1, as any queue operation does.
    pub(crate) fn share_session_queue(&mut self) -> Result<(), Failure> {
        if self.queue.sharing == Sharing::Local {
            match rxapi_client::client() {
                Ok(client) => {
                    move_lines(client, &mut self.queue.lines);
                    self.queue.sharing = Sharing::Remote;
                }
                Err(_) if self.queue.lines.is_empty() => return Ok(()),
                Err(status) => return Err(queue_failure(status).into()),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::BodyKey;
    use crate::{Activation, Interp};
    use interrexx_parse::{Program, parse_program};
    use std::rc::Rc;

    /// The module doc's own probe (`push "a"`, `queue "b"`, `push "c"`),
    /// asserted against the queue type directly rather than through the
    /// instruction loop.
    #[test]
    fn interleaved_push_and_queue_match_the_oracle_order() {
        let mut queue = Queue::new(false);
        queue.push(b"a".to_vec());
        queue.queue(b"b".to_vec());
        queue.push(b"c".to_vec());
        assert_eq!(
            queue.lines,
            VecDeque::from([b"c".to_vec(), b"a".to_vec(), b"b".to_vec()])
        );
    }

    /// A queue touched only by `QUEUE`, never `PUSH`, is a plain FIFO with no
    /// head insertion to get right.
    #[test]
    fn queue_alone_is_plain_fifo() {
        let mut queue = Queue::new(false);
        queue.queue(b"a".to_vec());
        queue.queue(b"b".to_vec());
        queue.queue(b"c".to_vec());
        assert_eq!(
            queue.lines,
            VecDeque::from([b"a".to_vec(), b"b".to_vec(), b"c".to_vec()])
        );
    }

    /// Which end [`Queue::pop`] takes from, which neither test above can see.
    #[test]
    fn interleaved_push_and_queue_survive_a_round_trip() {
        let mut queue = Queue::new(false);
        queue.push(b"a".to_vec());
        queue.queue(b"b".to_vec());
        queue.push(b"c".to_vec());
        assert_eq!(queue.pop().as_deref(), Some(&b"c"[..]));
        assert_eq!(queue.pop().as_deref(), Some(&b"a"[..]));
        assert_eq!(queue.pop().as_deref(), Some(&b"b"[..]));
        // Emptiness has to be distinguishable from a queued empty line, since
        // only the first sends `PULL` on to `.input`.
        assert_eq!(queue.pop(), None);
        queue.queue(Vec::new());
        assert_eq!(queue.pop().as_deref(), Some(&b""[..]));
        assert_eq!(queue.pop(), None);
    }

    /// Pushes a fresh top-level activation for `program`, the minimal setup
    /// `Interp::run` does. Copied rather than shared, matching every other
    /// test module in this crate -- `run.rs`'s own copy of this same
    /// function has why: `eval.rs`, `stem.rs` and `plan.rs` each keep their
    /// own rather than exporting one for every caller to share.
    fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
        let program = Rc::new(program);
        let program_id = interp.programs.insert(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: program_id,
                directive: None,
            },
            &program.main,
            &program.symbols,
            &program.source,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp.push_activation(Activation::new(
            id,
            Rc::clone(&program),
            program_id,
            plan,
            frame,
        ));
        program
    }

    /// The running interpreter writes into its queue: runs the three
    /// `PUSH`/`QUEUE` clauses through `Interp::run_activation` and inspects
    /// the queue afterward, since the type-level tests above cannot see
    /// whether `Interp::queue_evaluated` ever calls `Queue::push`/`Queue::queue`.
    #[test]
    fn push_and_queue_actually_write_into_the_running_interpreters_queue() {
        let mut interp = Interp::new();
        let program = parse_program(b"push \"a\"\nqueue \"b\"\npush \"c\"\n".to_vec())
            .expect("test program parses");
        activate(&mut interp, program);
        interp.run_activation().expect("push/queue never fail");
        assert_eq!(
            interp.queue.lines,
            VecDeque::from([b"c".to_vec(), b"a".to_vec(), b"b".to_vec()])
        );
    }
}
