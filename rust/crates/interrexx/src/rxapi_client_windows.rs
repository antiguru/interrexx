//! Named queues and macrospace require the external RXAPI service.
use std::path::Path;

use interrexx_rxapi::message::code::NO_SERVICE;
use interrexx_rxapi::message::{Reply, Request};
use interrexx_rxapi::transport::CancelToken;

pub(crate) enum ApiClient {}

/// Selecting a daemon requires a Windows RXAPI transport.
pub fn use_api_service(_dir: &Path, _daemon: &Path) -> bool {
    false
}

pub(crate) fn has_shared_session() -> bool {
    false
}

pub(crate) fn session_handle() -> u64 {
    u64::from(std::process::id())
}

pub(crate) fn client() -> Result<&'static ApiClient, u64> {
    Err(NO_SERVICE)
}

pub(crate) fn running_client() -> Option<&'static ApiClient> {
    None
}

impl ApiClient {
    pub(crate) fn session(&self) -> u64 {
        match *self {}
    }

    pub(crate) fn call(&self, _request: Request) -> Result<Reply, u64> {
        match *self {}
    }

    pub(crate) fn call_cancellable(
        &self,
        _request: Request,
        _token: &CancelToken,
    ) -> Result<Reply, u64> {
        match *self {}
    }

    pub(crate) fn cancel(&self, _token: &CancelToken) {
        match *self {}
    }
}
