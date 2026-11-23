//! The API service: external queues, the registries and the macrospace.

pub mod client;
pub mod core;
#[cfg(unix)]
pub mod daemon;
#[cfg(unix)]
mod descriptors;
#[cfg(unix)]
pub mod inheritance;
pub mod macro_file;
pub mod message;
#[cfg(unix)]
pub mod reserved;
#[cfg(unix)]
mod signals;
pub mod transport;
pub mod utility;
