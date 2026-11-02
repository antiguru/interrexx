//! How requests reach a `Core`.

#[cfg(any(test, feature = "test-direct"))]
pub mod direct;
pub mod unix;

use std::fmt;

use crate::message::code::NO_SERVICE;
use crate::message::{Reply, Request};

/// Carries one request to its reply, blocking the calling thread only.
pub trait Transport {
    fn call(&self, request: Request) -> Result<Reply, TransportError>;
}

/// Why a request got no reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportError {
    /// No service could be reached, or the connection to it is gone.
    Unavailable(String),
    /// The service speaks another protocol version.
    Refused { ours: u32, theirs: u32 },
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
        }
    }
}

impl std::error::Error for TransportError {}
