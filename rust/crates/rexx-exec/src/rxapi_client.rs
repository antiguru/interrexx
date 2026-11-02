//! The process's one connection to the API service, opened on first need and kept for the
//! process's life, so a registration it makes stays owned by it.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rexx_rxapi::client::{Session, connect};
use rexx_rxapi::message::code::NO_SERVICE;
use rexx_rxapi::message::wire::PROTOCOL_VERSION;
use rexx_rxapi::message::{Reply, Request};
use rexx_rxapi::transport::unix::{Unix, service_dir, socket_path};
use rexx_rxapi::transport::{CancelToken, Transport};

pub(crate) use rexx_rxapi::client::{SESSION_VARIABLE, session_value};

/// The connection and the session queue it serves.
pub(crate) struct ApiClient {
    transport: Unix,
    session: u64,
}

/// The connection, or the failure every later call answers.
static CLIENT: OnceLock<Result<ApiClient, u64>> = OnceLock::new();

/// The service directory and the daemon to start in it, where the process named them.
static SERVICE: OnceLock<(PathBuf, PathBuf)> = OnceLock::new();

/// Points this process's connection at the service in `dir`, started from `daemon`, instead of
/// the one `REXX_API_DIR` or the defaults name: for an embedding, such as a test harness, whose
/// programs must not reach the user's service. `false` where the process already chose one.
pub fn use_api_service(dir: &Path, daemon: &Path) -> bool {
    CLIENT.get().is_none()
        && SERVICE
            .set((dir.to_path_buf(), daemon.to_path_buf()))
            .is_ok()
}

/// The service directory this process uses.
fn chosen_dir() -> PathBuf {
    match SERVICE.get() {
        Some((dir, _)) => dir.clone(),
        None => service_dir(),
    }
}

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
        .get_or_init(|| crate::signal::blocked(connect_client))
        .as_ref()
        .map_err(|status| *status)
}

/// A new connection, attached to this process's session queue.
fn connect_client() -> Result<ApiClient, u64> {
    if !connections_allowed() {
        return Err(NO_SERVICE);
    }
    let transport = match SERVICE.get() {
        Some((dir, daemon)) => {
            Unix::connect_or_start_with(dir, daemon).map_err(|error| error.status())?
        }
        None => connect()?,
    };
    let client = ApiClient {
        transport,
        session: session_handle(),
    };
    client.call(Request::CreateSessionQueue {
        session: client.session,
    })?;
    Ok(client)
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
    let dir = chosen_dir();
    if !socket_path(&dir, PROTOCOL_VERSION).exists() {
        return None;
    }
    let transport =
        crate::signal::blocked(|| Unix::connect_version(&dir, PROTOCOL_VERSION)).ok()?;
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
/// directory, so none reaches the user's own.
fn connections_allowed() -> bool {
    !cfg!(test) || SERVICE.get().is_some() || std::env::var_os("REXX_API_DIR").is_some()
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
