//! The process's one connection to the API service, opened on first need and kept for the
//! process's life, so a registration it makes stays owned by it.

use std::sync::OnceLock;

use rexx_rxapi::client::{Session, connect};
use rexx_rxapi::message::code::NO_SERVICE;
use rexx_rxapi::message::wire::PROTOCOL_VERSION;
use rexx_rxapi::message::{Reply, Request};
use rexx_rxapi::transport::unix::{Unix, service_dir};
use rexx_rxapi::transport::{CancelToken, Transport};

pub(crate) use rexx_rxapi::client::{SESSION_VARIABLE, session_value};

/// The connection and the session queue it serves.
pub(crate) struct ApiClient {
    transport: Unix,
    session: u64,
}

/// The connection, or the failure every later call answers.
static CLIENT: OnceLock<Result<ApiClient, u64>> = OnceLock::new();

/// This process's session queue.
pub(crate) fn session() -> Session {
    static SESSION: OnceLock<Session> = OnceLock::new();
    *SESSION.get_or_init(Session::of_process)
}

/// This process's session handle.
pub(crate) fn session_handle() -> u64 {
    session().handle()
}

/// The process's client, connecting and starting the service on first use. A failure is the
/// API code, and stays the answer for the process's life.
pub(crate) fn client() -> Result<&'static ApiClient, u64> {
    CLIENT
        .get_or_init(|| {
            if !connections_allowed() {
                return Err(NO_SERVICE);
            }
            let client = ApiClient {
                transport: connect()?,
                session: session_handle(),
            };
            client.call(Request::CreateSessionQueue {
                session: client.session,
            })?;
            Ok(client)
        })
        .as_ref()
        .map_err(|status| *status)
}

/// The process's client where it is connected or a service is running to connect to, starting
/// none. An absent service is not remembered as a failure.
pub(crate) fn running_client() -> Option<&'static ApiClient> {
    if let Some(client) = CLIENT.get() {
        return client.as_ref().ok();
    }
    if !connections_allowed() {
        return None;
    }
    let transport = Unix::connect_version(&service_dir(), PROTOCOL_VERSION).ok()?;
    let client = ApiClient {
        transport,
        session: session_handle(),
    };
    client
        .call(Request::CreateSessionQueue {
            session: client.session,
        })
        .ok()?;
    let _ = CLIENT.set(Ok(client));
    CLIENT.get()?.as_ref().ok()
}

/// Whether this process may reach a service: a unit test may only through a private
/// `REXX_API_DIR`, so none reaches the user's own.
fn connections_allowed() -> bool {
    !cfg!(test) || std::env::var_os("REXX_API_DIR").is_some()
}

impl ApiClient {
    pub(crate) fn session(&self) -> u64 {
        self.session
    }

    /// One request's reply, or `RXAPI_NORXAPI` when the service is gone.
    pub(crate) fn call(&self, request: Request) -> Result<Reply, u64> {
        self.transport.call(request).map_err(|_| NO_SERVICE)
    }

    /// As [`ApiClient::call`], ended early by `token`.
    pub(crate) fn call_cancellable(
        &self,
        request: Request,
        token: &CancelToken,
    ) -> Result<Reply, u64> {
        self.transport
            .call_cancellable(request, token)
            .map_err(|_| NO_SERVICE)
    }

    /// Ends the call `token` started, from any thread.
    pub(crate) fn cancel(&self, token: &CancelToken) {
        let _ = self.transport.cancel(token);
    }
}
