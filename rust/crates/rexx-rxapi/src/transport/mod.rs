//! How requests reach a `Core`.

#[cfg(any(test, feature = "test-direct"))]
pub mod direct;
pub mod unix;

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::message::code::NO_SERVICE;
use crate::message::{Reply, Request};

/// Carries one request to its reply, blocking the calling thread only.
pub trait Transport {
    fn call(&self, request: Request) -> Result<Reply, TransportError> {
        self.call_cancellable(request, &CancelToken::default())
    }

    /// As `call`. Once `token` is cancelled, a pull not yet answered answers `Reply::Cancelled`,
    /// and a call not yet started does not start.
    fn call_cancellable(
        &self,
        request: Request,
        token: &CancelToken,
    ) -> Result<Reply, TransportError>;

    /// Cancels `token` from any thread, leaving the connection open.
    fn cancel(&self, token: &CancelToken) -> Result<(), TransportError> {
        match token.cancel() {
            Some(call) => self.call(Request::CancelPull { call }).map(drop),
            None => Ok(()),
        }
    }
}

/// Lets one thread end the call another is blocked in.
#[derive(Clone, Default)]
pub struct CancelToken(Arc<Mutex<Started>>);

/// A token's call. An implementor holds this lock across issuing the call, so a cancel always
/// reaches the service after the call it cancels.
#[derive(Default)]
struct Started {
    cancelled: bool,
    call: Option<u64>,
}

impl CancelToken {
    fn started(&self) -> MutexGuard<'_, Started> {
        self.0.lock().expect("the critical sections cannot panic")
    }

    fn cancel(&self) -> Option<u64> {
        let mut started = self.started();
        started.cancelled = true;
        started.call
    }
}

/// Why a request got no reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportError {
    /// No service could be reached, or the connection to it is gone.
    Unavailable(String),
    /// The service speaks another protocol version.
    Refused { ours: u32, theirs: u32 },
    /// The service belongs to another user.
    Foreign { uid: u32 },
}

impl TransportError {
    /// The API return code for this failure.
    pub fn status(&self) -> u64 {
        NO_SERVICE
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::Unavailable(reason) => write!(formatter, "no service: {reason}"),
            TransportError::Refused { ours, theirs } => {
                write!(
                    formatter,
                    "service protocol {theirs}, client protocol {ours}"
                )
            }
            TransportError::Foreign { uid } => write!(formatter, "service run by user {uid}"),
        }
    }
}

impl std::error::Error for TransportError {}
