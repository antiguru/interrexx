//! Exercises the Windows pipe client against a separately started rxapi process.
#![cfg(windows)]

use interrexx_rxapi::message::{Reply, Request, WAIT, wire::PROTOCOL_VERSION};
use interrexx_rxapi::transport::{CancelToken, Transport, windows::Windows};
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::Duration;

struct Service(PathBuf);

impl Drop for Service {
    fn drop(&mut self) {
        let _ = Windows::stop(&self.0);
    }
}

#[test]
fn starts_a_private_service_and_cancels_a_pull() {
    let base = std::env::temp_dir().join(format!("interrexx-pipe-test-{}", std::process::id()));
    let daemon = Path::new(env!("CARGO_BIN_EXE_rxapi"));
    let service = Service(base.join("service"));
    let client = Arc::new(Windows::connect_or_start_with(&service.0, daemon).expect("start rxapi"));
    assert!(Windows::connect_version(&base.join("isolated"), PROTOCOL_VERSION).is_err());

    let name = Some(b"CANCELQ".to_vec());
    assert!(
        client
            .call(Request::CreateQueue {
                session: 1,
                requested: name.clone(),
                size: 128,
            })
            .is_ok()
    );

    let token = CancelToken::default();
    let pulling = Arc::clone(&client);
    let pull_token = token.clone();
    let (send, receive) = mpsc::channel();
    let pull_name = name.clone();
    let reader = std::thread::spawn(move || {
        let reply = pulling.call_cancellable(
            Request::PullQueue {
                session: 1,
                name: pull_name,
                wait: WAIT,
            },
            &pull_token,
        );
        let _ = send.send(reply);
    });
    client.cancel(&token).expect("queue cancellation");
    assert_eq!(
        receive
            .recv_timeout(Duration::from_secs(3))
            .expect("the pull woke"),
        Ok(Reply::Cancelled),
    );
    reader.join().expect("the reader exited");
    assert!(client.call(Request::DeleteQueue { name }).is_ok());
}
