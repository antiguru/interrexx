//! Exercises the Windows pipe client against a separately started rxapi process.
#![cfg(windows)]

use interrexx_rxapi::message::code::{OK, QUEUE_ACCESS, QUEUE_NOT_REGISTERED};
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

/// The daemon refuses to delete a named queue while a pull is parked on it.
#[expect(clippy::disallowed_methods, reason = "polling a real Windows daemon")]
fn wait_until_parked(client: &Windows, name: &Option<Vec<u8>>) {
    for _ in 0..400 {
        match client
            .call(Request::DeleteQueue { name: name.clone() })
            .expect("probe the daemon")
        {
            Reply::Status(QUEUE_ACCESS) => return,
            Reply::Status(OK) => {
                assert!(matches!(
                    client.call(Request::OpenQueue { name: name.clone() }),
                    Ok(Reply::QueueOpened { status: OK, .. })
                ));
            }
            reply => panic!("unexpected delete probe reply: {reply:?}"),
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("the daemon never parked the pull");
}

#[test]
fn starts_a_private_service_and_cancels_a_pull() {
    let base = std::env::temp_dir().join(format!("interrexx-pipe-test-{}", std::process::id()));
    let daemon = Path::new(env!("CARGO_BIN_EXE_rxapi"));
    let service = Service(base.join("service"));
    let client = Arc::new(Windows::connect_or_start_with(&service.0, daemon).expect("start rxapi"));
    assert!(Windows::connect_version(&base.join("isolated"), PROTOCOL_VERSION).is_err());

    let name = Some(b"CANCELQ".to_vec());
    assert!(matches!(
        client.call(Request::CreateQueue {
            session: 1,
            requested: name.clone(),
            size: 128,
        }),
        Ok(Reply::QueueCreated { status: OK, .. })
    ));

    let token = CancelToken::default();
    let pulling = Arc::clone(&client);
    let pull_token = token.clone();
    let (send, receive) = mpsc::channel();
    let pull_name = name.clone();
    let reader = std::thread::spawn(move || {
        let reply = loop {
            let reply = pulling.call_cancellable(
                Request::PullQueue {
                    session: 1,
                    name: pull_name.clone(),
                    wait: WAIT,
                },
                &pull_token,
            );
            // The probe can delete the queue just before this pull arrives.
            // Only retry that gap while the probing thread reopens the queue.
            if !matches!(
                reply,
                Ok(Reply::Pulled {
                    status: QUEUE_NOT_REGISTERED,
                    item: None,
                })
            ) {
                break reply;
            }
        };
        let _ = send.send(reply);
    });
    wait_until_parked(&client, &name);
    client.cancel(&token).expect("queue cancellation");
    assert_eq!(
        receive
            .recv_timeout(Duration::from_secs(3))
            .expect("the pull woke"),
        Ok(Reply::Cancelled),
    );
    reader.join().expect("the reader exited");
    assert_eq!(
        client.call(Request::DeleteQueue { name }),
        Ok(Reply::Status(OK))
    );
}
