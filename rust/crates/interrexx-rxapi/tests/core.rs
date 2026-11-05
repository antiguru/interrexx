//! The core, in process: one test per section of `rxapi-behaviour.md`, by its values.

use interrexx_rxapi::core::{ClientId, Core, Outcome};
use interrexx_rxapi::message::code::*;
use interrexx_rxapi::message::{
    FIFO, Item, LIFO, Macro, NO_WAIT, Registered, Registry, Reply, Request, Timestamp, WAIT,
};

const SESSION: u64 = 0x1f4a5f;

fn client(id: u64) -> ClientId {
    ClientId::connection(id)
}

fn name(text: &str) -> Option<Vec<u8>> {
    Some(text.as_bytes().to_vec())
}

fn reply(core: &mut Core, client: ClientId, request: Request) -> Reply {
    match core.handle(client, request) {
        Outcome::Reply(reply) => reply,
        Outcome::Parked => panic!("parked unexpectedly"),
    }
}

fn status(core: &mut Core, client: ClientId, request: Request) -> u64 {
    match reply(core, client, request) {
        Reply::Status(status)
        | Reply::QueueCreated { status, .. }
        | Reply::QueueOpened { status, .. }
        | Reply::QueueCount { status, .. }
        | Reply::Pulled { status, .. }
        | Reply::Registration { status, .. }
        | Reply::MacroPosition { status, .. }
        | Reply::MacroImage { status, .. }
        | Reply::Macros { status, .. } => status,
        Reply::Cancelled => panic!("a cancel is not a status"),
    }
}

fn open(core: &mut Core, queue: &str) -> Reply {
    reply(core, client(1), Request::OpenQueue { name: name(queue) })
}

fn add(queue: Option<&str>, data: &str, flag: u64) -> Request {
    Request::AddQueue {
        session: SESSION,
        name: queue.and_then(name),
        data: data.as_bytes().to_vec(),
        flag,
        time: Timestamp::default(),
    }
}

fn pull(queue: Option<&str>, wait: u64) -> Request {
    Request::PullQueue {
        session: SESSION,
        name: queue.and_then(name),
        wait,
    }
}

fn pulled(core: &mut Core, queue: Option<&str>) -> Vec<u8> {
    match reply(core, client(1), pull(queue, NO_WAIT)) {
        Reply::Pulled {
            status: OK,
            item: Some(item),
        } => item.data,
        other => panic!("no item: {other:?}"),
    }
}

fn count(core: &mut Core, queue: Option<&str>) -> Reply {
    reply(
        core,
        client(1),
        Request::QueryQueue {
            session: SESSION,
            name: queue.and_then(name),
        },
    )
}

fn create(core: &mut Core, requested: Option<&str>, size: u64) -> Reply {
    reply(
        core,
        client(1),
        Request::CreateQueue {
            session: SESSION,
            requested: requested.and_then(name),
            size,
        },
    )
}

fn delete(core: &mut Core, queue: Option<&str>) -> u64 {
    status(
        core,
        client(1),
        Request::DeleteQueue {
            name: queue.and_then(name),
        },
    )
}

fn clear(core: &mut Core, queue: Option<&str>) -> u64 {
    status(
        core,
        client(1),
        Request::ClearQueue {
            session: SESSION,
            name: queue.and_then(name),
        },
    )
}

fn exists(core: &mut Core, queue: Option<&str>) -> u64 {
    status(
        core,
        client(1),
        Request::QueueExists {
            name: queue.and_then(name),
        },
    )
}

#[test]
fn queue_names_are_validated() {
    let mut core = Core::new();
    let longest = "_".repeat(250);
    for valid in ["0", "?", "!", ".", "QUEUE", "QUEUE.0", longest.as_str()] {
        assert_eq!(
            open(&mut core, valid),
            Reply::QueueOpened {
                status: OK,
                duplicate: Some(false)
            },
            "{valid}"
        );
    }
    let long = "_".repeat(251);
    for invalid in ["", "a b", "a-b", "é", long.as_str()] {
        assert_eq!(
            open(&mut core, invalid),
            Reply::QueueOpened {
                status: QUEUE_BAD_NAME,
                duplicate: None
            },
            "{invalid}"
        );
    }
}

#[test]
fn session_and_null_names_follow_the_table() {
    let mut core = Core::new();
    assert_eq!(
        create(&mut core, Some("session"), 100),
        Reply::QueueCreated {
            status: QUEUE_BAD_NAME,
            name: None,
            duplicate: None
        }
    );
    assert_eq!(
        open(&mut core, "SESSION"),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(true)
        }
    );
    assert_eq!(exists(&mut core, Some("Session")), OK);
    assert_eq!(delete(&mut core, Some("SESSION")), QUEUE_BAD_NAME);
    assert!(core.is_empty(), "SESSION opened nothing");

    assert_eq!(exists(&mut core, None), QUEUE_NOT_REGISTERED);
    assert_eq!(
        reply(&mut core, client(1), Request::OpenQueue { name: None }),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(false)
        }
    );
    assert_eq!(exists(&mut core, None), OK);
    assert_eq!(
        reply(&mut core, client(1), Request::OpenQueue { name: None }),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(true)
        }
    );
    assert_eq!(delete(&mut core, None), OK);
    assert_eq!(exists(&mut core, None), QUEUE_NOT_REGISTERED);

    assert_eq!(status(&mut core, client(1), add(None, "a", FIFO)), OK);
    assert_eq!(
        status(&mut core, client(1), add(Some("session"), "b", FIFO)),
        OK
    );
    assert_eq!(
        count(&mut core, Some("SESSION")),
        Reply::QueueCount {
            status: OK,
            count: Some(2)
        }
    );
    assert_eq!(pulled(&mut core, None), b"a");
    assert_eq!(pulled(&mut core, Some("SESSION")), b"b");
    assert_eq!(clear(&mut core, Some("SESSION")), OK);
    assert_eq!(clear(&mut core, None), OK);
}

#[test]
fn create_queue_checks_buffer_then_name_then_existence() {
    let mut core = Core::new();
    assert_eq!(
        create(&mut core, Some("bad name!!"), 5),
        Reply::QueueCreated {
            status: QUEUE_STORAGE,
            name: None,
            duplicate: None
        },
        "the buffer check wins over validation"
    );
    assert_eq!(
        create(&mut core, Some("abcde"), 5),
        Reply::QueueCreated {
            status: QUEUE_STORAGE,
            name: None,
            duplicate: None
        },
        "strlen == size does not fit"
    );
    assert!(core.is_empty());
    assert_eq!(
        create(&mut core, Some("abcd"), 5),
        Reply::QueueCreated {
            status: OK,
            name: name("abcd"),
            duplicate: Some(false)
        },
        "strlen == size - 1 fits"
    );
    assert_eq!(
        create(&mut core, Some("a b"), 100),
        Reply::QueueCreated {
            status: QUEUE_BAD_NAME,
            name: None,
            duplicate: None
        }
    );
    assert_eq!(
        create(&mut core, Some("myQueue"), 100),
        Reply::QueueCreated {
            status: OK,
            name: name("myQueue"),
            duplicate: Some(false)
        },
        "the requested spelling is kept"
    );
    let Reply::QueueCreated {
        status: OK,
        name: Some(generated),
        duplicate: Some(true),
    } = create(&mut core, Some("MYQUEUE"), 100)
    else {
        panic!("an existing name makes a generated-name queue");
    };
    assert_ne!(generated, b"MYQUEUE");
    assert_eq!(exists(&mut core, std::str::from_utf8(&generated).ok()), OK);
}

#[test]
fn generated_names_are_valid_unique_and_truncate_to_the_buffer() {
    let mut core = Core::new();
    let mut seen = std::collections::HashSet::new();
    for _ in 0..100 {
        let Reply::QueueCreated {
            status: OK,
            name: Some(generated),
            duplicate: Some(false),
        } = create(&mut core, None, 100)
        else {
            panic!("a generated create");
        };
        assert_eq!(generated[0], b'S');
        assert!(generated.len() >= 10);
        assert!(generated.iter().all(u8::is_ascii_alphanumeric));
        assert!(seen.insert(generated));
    }

    let mut core = Core::new();
    let Reply::QueueCreated {
        status: QUEUE_STORAGE,
        name: Some(truncated),
        duplicate: Some(false),
    } = create(&mut core, None, 10)
    else {
        panic!("a generated name of at least 10 characters does not fit in 10 bytes");
    };
    assert_eq!(truncated.len(), 9);
    assert!(!core.is_empty(), "the queue has still been created");

    assert_eq!(
        create(&mut core, None, 0),
        Reply::QueueCreated {
            status: QUEUE_STORAGE,
            name: Some(Vec::new()),
            duplicate: Some(false)
        }
    );
}

#[test]
fn open_exists_and_delete() {
    let mut core = Core::new();
    assert_eq!(exists(&mut core, Some("q")), QUEUE_NOT_REGISTERED);
    assert_eq!(exists(&mut core, Some("a b")), QUEUE_BAD_NAME);
    assert_eq!(
        open(&mut core, "q"),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(false)
        }
    );
    assert_eq!(
        open(&mut core, "Q"),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(true)
        }
    );
    assert_eq!(exists(&mut core, Some("Q")), OK);

    assert_eq!(delete(&mut core, Some("a b")), QUEUE_BAD_NAME);
    assert_eq!(delete(&mut core, Some("missing")), QUEUE_NOT_REGISTERED);
    assert_eq!(
        status(&mut core, client(1), add(Some("q"), "item", FIFO)),
        OK
    );
    assert_eq!(
        delete(&mut core, Some("q")),
        OK,
        "deletes a queue with items"
    );
    assert_eq!(exists(&mut core, Some("q")), QUEUE_NOT_REGISTERED);
}

#[test]
fn clear_and_query() {
    let mut core = Core::new();
    assert_eq!(clear(&mut core, Some("a b")), QUEUE_BAD_NAME);
    assert_eq!(clear(&mut core, Some("q")), QUEUE_NOT_REGISTERED);
    open(&mut core, "q");
    assert_eq!(clear(&mut core, Some("q")), OK, "nothing to clear");
    status(&mut core, client(1), add(Some("q"), "a", FIFO));
    assert_eq!(
        count(&mut core, Some("q")),
        Reply::QueueCount {
            status: OK,
            count: Some(1)
        }
    );
    assert_eq!(clear(&mut core, Some("q")), OK);
    assert_eq!(
        count(&mut core, Some("q")),
        Reply::QueueCount {
            status: OK,
            count: Some(0)
        }
    );
    assert_eq!(
        count(&mut core, Some("missing")),
        Reply::QueueCount {
            status: QUEUE_NOT_REGISTERED,
            count: Some(0)
        }
    );
    assert_eq!(
        count(&mut core, Some("a b")),
        Reply::QueueCount {
            status: QUEUE_BAD_NAME,
            count: None
        }
    );
}

#[test]
fn add_checks_flag_first_and_orders_by_head() {
    let mut core = Core::new();
    assert_eq!(
        status(&mut core, client(1), add(Some("a b"), "x", 2)),
        QUEUE_PRIORITY
    );
    assert_eq!(
        status(&mut core, client(1), add(Some("missing"), "x", 2)),
        QUEUE_PRIORITY
    );
    assert_eq!(
        status(&mut core, client(1), add(Some("a b"), "x", FIFO)),
        QUEUE_BAD_NAME
    );
    assert_eq!(
        status(&mut core, client(1), add(Some("missing"), "x", FIFO)),
        QUEUE_NOT_REGISTERED
    );

    open(&mut core, "q");
    for (item, flag) in [
        ("a", FIFO),
        ("b", FIFO),
        ("c", LIFO),
        ("d", LIFO),
        ("e", FIFO),
    ] {
        assert_eq!(status(&mut core, client(1), add(Some("q"), item, flag)), OK);
    }
    let order: Vec<Vec<u8>> = (0..5).map(|_| pulled(&mut core, Some("q"))).collect();
    assert_eq!(order, [b"d", b"c", b"a", b"b", b"e"]);
}

#[test]
fn items_are_bytes_and_keep_their_timestamp() {
    let mut core = Core::new();
    open(&mut core, "q");
    let data = vec![0u8, 255, 0, b'a'];
    let time = Timestamp {
        year: 126,
        month: 10,
        day: 2,
        weekday: 1,
        yearday: 305,
        hours: 12,
        minutes: 30,
        seconds: 5,
    };
    for data in [data.clone(), Vec::new()] {
        let request = Request::AddQueue {
            session: SESSION,
            name: name("q"),
            data: data.clone(),
            flag: FIFO,
            time,
        };
        assert_eq!(status(&mut core, client(1), request), OK);
        assert_eq!(
            reply(&mut core, client(1), pull(Some("q"), NO_WAIT)),
            Reply::Pulled {
                status: OK,
                item: Some(Item { data, time })
            }
        );
    }
}

#[test]
fn pull_checks_flag_first_and_answers_empty() {
    let mut core = Core::new();
    assert_eq!(
        status(&mut core, client(1), pull(Some("a b"), 2)),
        QUEUE_BAD_WAIT_FLAG
    );
    assert_eq!(
        reply(&mut core, client(1), pull(Some("a b"), NO_WAIT)),
        Reply::Pulled {
            status: QUEUE_BAD_NAME,
            item: None
        }
    );
    assert_eq!(
        reply(&mut core, client(1), pull(Some("missing"), WAIT)),
        Reply::Pulled {
            status: QUEUE_NOT_REGISTERED,
            item: None
        },
        "a missing queue never waits"
    );
    open(&mut core, "q");
    assert_eq!(
        reply(&mut core, client(1), pull(Some("q"), NO_WAIT)),
        Reply::Pulled {
            status: QUEUE_EMPTY,
            item: None
        }
    );
}

#[test]
fn a_waiting_pull_gets_one_item_and_blocks_delete() {
    let mut core = Core::new();
    open(&mut core, "q");
    status(&mut core, client(1), add(Some("q"), "ready", FIFO));
    assert!(
        matches!(
            core.handle(client(2), pull(Some("q"), WAIT)),
            Outcome::Reply(Reply::Pulled { status: OK, .. })
        ),
        "an item already queued is returned at once"
    );

    assert_eq!(
        core.handle(client(2), pull(Some("q"), WAIT)),
        Outcome::Parked
    );
    assert_eq!(
        core.handle(client(3), pull(Some("q"), WAIT)),
        Outcome::Parked
    );
    assert!(core.take_ready().is_empty());

    assert_eq!(delete(&mut core, Some("q")), QUEUE_ACCESS);
    assert_eq!(
        clear(&mut core, Some("q")),
        OK,
        "clear leaves waiters waiting"
    );

    assert_eq!(status(&mut core, client(1), add(Some("q"), "x", LIFO)), OK);
    let first = core.take_ready();
    assert_eq!(first.len(), 1, "an add satisfies one waiter");
    assert!(core.take_ready().is_empty(), "take_ready drains");
    assert_eq!(delete(&mut core, Some("q")), QUEUE_ACCESS);
    assert_eq!(status(&mut core, client(1), add(Some("q"), "y", FIFO)), OK);
    let second = core.take_ready();
    assert_eq!(second.len(), 1);
    assert_ne!(first[0].0, second[0].0, "each item to exactly one waiter");
    assert_eq!(
        count(&mut core, Some("q")),
        Reply::QueueCount {
            status: OK,
            count: Some(0)
        }
    );
    assert_eq!(
        delete(&mut core, Some("q")),
        OK,
        "deletable again once the waiters have their items"
    );
}

#[test]
fn two_parked_pulls_of_one_connection_are_told_apart() {
    let mut core = Core::new();
    open(&mut core, "q");
    let (one, two) = (client(2).call(1), client(2).call(2));
    assert_eq!(core.handle(one, pull(Some("q"), WAIT)), Outcome::Parked);
    assert_eq!(core.handle(two, pull(Some("q"), WAIT)), Outcome::Parked);
    status(&mut core, client(1), add(Some("q"), "a", FIFO));
    status(&mut core, client(1), add(Some("q"), "b", FIFO));
    let ready: Vec<ClientId> = core.take_ready().into_iter().map(|(id, _)| id).collect();
    assert_eq!(ready.len(), 2);
    assert!(ready.contains(&one) && ready.contains(&two));
}

/// Review Focus 2, at this level: a killed blocked client is one whose connection closed.
#[test]
fn a_killed_blocked_client_leaves_its_queue_deletable() {
    let mut core = Core::new();
    open(&mut core, "q");
    assert_eq!(
        core.handle(client(2), pull(Some("q"), WAIT)),
        Outcome::Parked
    );
    assert_eq!(delete(&mut core, Some("q")), QUEUE_ACCESS);

    core.client_closed(client(2));
    assert!(
        core.take_ready().is_empty(),
        "the waiter is simply forgotten"
    );

    status(&mut core, client(1), add(Some("q"), "kept", FIFO));
    assert!(core.take_ready().is_empty(), "no reply to a closed client");
    assert_eq!(
        count(&mut core, Some("q")),
        Reply::QueueCount {
            status: OK,
            count: Some(1)
        },
        "the next item stays in the queue"
    );
    assert_eq!(delete(&mut core, Some("q")), OK);
}

const MEBIBYTE: usize = 1 << 20;

fn sized_macro(core: &mut Core, macro_name: String, size: usize) {
    let request = Request::AddMacro {
        name: macro_name.into_bytes(),
        image: vec![0; size],
        position: 1,
    };
    assert_eq!(status(core, client(1), request), OK);
}

fn save_names(core: &mut Core, names: Option<Vec<Vec<u8>>>) -> Reply {
    reply(core, client(1), Request::SaveMacros { names })
}

#[test]
fn a_save_repeating_a_name_beyond_the_limit_is_refused_before_it_is_built() {
    let mut core = Core::new();
    sized_macro(&mut core, "big".to_owned(), MEBIBYTE);
    let repeated = vec![b"big".to_vec(); 512];

    assert_eq!(
        save_names(&mut core, Some(repeated.clone())),
        Reply::Macros {
            status: MACRO_NO_STORAGE,
            macros: Vec::new()
        }
    );

    let mut with_absent = repeated;
    with_absent.push(b"absent".to_vec());
    assert_eq!(
        save_names(&mut core, Some(with_absent)),
        Reply::Macros {
            status: MACRO_NOT_FOUND,
            macros: Vec::new()
        },
        "an absent name still answers first"
    );
    let Reply::Macros { status: OK, macros } =
        save_names(&mut core, Some(vec![b"big".to_vec(); 2]))
    else {
        panic!("a small repeat is a valid save");
    };
    assert_eq!(macros.len(), 2);
}

#[test]
fn a_macrospace_larger_than_the_limit_still_saves_whole() {
    let mut core = Core::new();
    let names: Vec<String> = (0..300).map(|at| format!("M{at}")).collect();
    for macro_name in &names {
        sized_macro(&mut core, macro_name.clone(), MEBIBYTE);
    }
    let listed: Vec<Vec<u8>> = names.iter().map(|name| name.as_bytes().to_vec()).collect();

    for selection in [None, Some(listed)] {
        let Reply::Macros { status: OK, macros } = save_names(&mut core, selection) else {
            panic!("every macro once is what a macrospace holds");
        };
        assert_eq!(macros.len(), names.len());
    }
}

/// Measured here: a list scanned per macro took 27 s, a set 0.04 s.
#[test]
fn a_selective_load_does_not_scan_the_names_per_macro() {
    let name_of = |at: usize| format!("MACRO{at:06}").into_bytes();
    let names: Vec<Vec<u8>> = (0..120_000).map(|at| name_of(at * 2 + 1)).collect();
    let macros: Vec<Macro> = (0..120_000)
        .map(|at| Macro {
            name: name_of(at * 2),
            image: Vec::new(),
            position: 1,
        })
        .collect();
    let mut core = Core::new();

    #[expect(clippy::disallowed_methods, reason = "this test is about real cost")]
    let start = std::time::Instant::now();
    let loaded = status(
        &mut core,
        client(1),
        Request::LoadMacros {
            names: Some(names),
            macros,
        },
    );
    #[expect(clippy::disallowed_methods, reason = "this test is about real cost")]
    let spent = start.elapsed();

    assert_eq!(loaded, OK);
    assert!(core.is_empty(), "no listed name was in the file");
    assert!(spent.as_secs() < 5, "{spent:?}");
}

fn cancel(core: &mut Core, connection: u64, call: u64) -> Reply {
    reply(core, client(connection), Request::CancelPull { call })
}

#[test]
fn a_cancelled_pull_leaves_the_queue_and_takes_no_later_item() {
    let mut core = Core::new();
    open(&mut core, "q");
    let parked = client(2).call(7);
    assert_eq!(core.handle(parked, pull(Some("q"), WAIT)), Outcome::Parked);
    assert_eq!(delete(&mut core, Some("q")), QUEUE_ACCESS);

    assert_eq!(cancel(&mut core, 2, 7), Reply::Status(OK));

    assert_eq!(core.take_ready(), [(parked, Reply::Cancelled)]);
    status(&mut core, client(1), add(Some("q"), "kept", FIFO));
    assert!(core.take_ready().is_empty(), "it takes no later item");
    assert_eq!(
        count(&mut core, Some("q")),
        Reply::QueueCount {
            status: OK,
            count: Some(1)
        }
    );
    assert_eq!(delete(&mut core, Some("q")), OK);
}

#[test]
fn a_cancel_works_on_a_session_queue() {
    let mut core = Core::new();
    let parked = client(2).call(1);
    assert_eq!(core.handle(parked, pull(None, WAIT)), Outcome::Parked);

    assert_eq!(cancel(&mut core, 2, 1), Reply::Status(OK));

    assert_eq!(core.take_ready(), [(parked, Reply::Cancelled)]);
}

#[test]
fn a_cancel_ends_only_that_call_of_that_connection() {
    let mut core = Core::new();
    open(&mut core, "q");
    let (one, two, other) = (client(2).call(1), client(2).call(2), client(3).call(1));
    for waiter in [one, two, other] {
        assert_eq!(core.handle(waiter, pull(Some("q"), WAIT)), Outcome::Parked);
    }

    assert_eq!(cancel(&mut core, 2, 1), Reply::Status(OK));
    assert_eq!(cancel(&mut core, 2, 99), Reply::Status(OK), "no such call");
    assert_eq!(cancel(&mut core, 4, 1), Reply::Status(OK), "not its call");

    assert_eq!(core.take_ready(), [(one, Reply::Cancelled)]);
    status(&mut core, client(1), add(Some("q"), "a", FIFO));
    status(&mut core, client(1), add(Some("q"), "b", FIFO));
    let ready: Vec<ClientId> = core.take_ready().into_iter().map(|(id, _)| id).collect();
    assert_eq!(ready, [two, other]);
}

fn item(data: &str) -> Item {
    Item {
        data: data.as_bytes().to_vec(),
        time: Timestamp::default(),
    }
}

#[test]
fn an_undelivered_item_goes_to_the_head_and_keeps_its_time() {
    let mut core = Core::new();
    open(&mut core, "q");
    status(&mut core, client(1), add(Some("q"), "later", FIFO));
    let stamped = Item {
        time: Timestamp {
            year: 126,
            ..Timestamp::default()
        },
        ..item("first")
    };

    core.undelivered(SESSION, name("Q"), stamped.clone());

    assert_eq!(
        reply(&mut core, client(1), pull(Some("q"), NO_WAIT)),
        Reply::Pulled {
            status: OK,
            item: Some(stamped)
        },
        "names fold, and the item keeps its place and its timestamp"
    );
    assert_eq!(pulled(&mut core, Some("q")), b"later");
}

#[test]
fn an_undelivered_item_goes_to_the_next_waiter() {
    let mut core = Core::new();
    open(&mut core, "q");
    assert_eq!(
        core.handle(client(2), pull(Some("q"), WAIT)),
        Outcome::Parked
    );

    core.undelivered(SESSION, name("q"), item("a"));

    let ready = core.take_ready();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].0, client(2));
    assert_eq!(
        count(&mut core, Some("q")),
        Reply::QueueCount {
            status: OK,
            count: Some(0)
        }
    );
}

#[test]
fn an_undelivered_item_of_a_session_queue_returns_to_that_queue() {
    let mut core = Core::new();
    status(&mut core, client(1), add(None, "b", FIFO));

    core.undelivered(SESSION, None, item("a"));

    assert_eq!(pulled(&mut core, None), b"a");
    assert_eq!(pulled(&mut core, Some("SESSION")), b"b");
}

#[test]
fn an_undelivered_item_of_a_deleted_queue_is_gone() {
    let mut core = Core::new();
    open(&mut core, "q");
    assert_eq!(delete(&mut core, Some("q")), OK);

    core.undelivered(SESSION, name("q"), item("a"));
    core.undelivered(SESSION + 1, None, item("b"));
    core.undelivered(SESSION, name("bad name"), item("c"));

    assert_eq!(exists(&mut core, Some("q")), QUEUE_NOT_REGISTERED);
    assert!(core.take_ready().is_empty());
    assert!(core.is_empty());
}

#[test]
fn closing_a_connection_ends_all_its_waits_and_nobody_elses() {
    let mut core = Core::new();
    open(&mut core, "q");
    let other = client(3);
    assert_eq!(
        core.handle(client(2).call(1), pull(Some("q"), WAIT)),
        Outcome::Parked
    );
    assert_eq!(
        core.handle(client(2).call(2), pull(Some("q"), WAIT)),
        Outcome::Parked
    );
    assert_eq!(core.handle(other, pull(Some("q"), WAIT)), Outcome::Parked);
    core.client_closed(client(2));
    assert_eq!(delete(&mut core, Some("q")), QUEUE_ACCESS);
    status(&mut core, client(1), add(Some("q"), "a", FIFO));
    let ready = core.take_ready();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].0, other);
}

#[test]
fn session_queues_are_shared_by_handle_and_on_demand() {
    let mut core = Core::new();
    for session in [1, 2] {
        let request = Request::AddQueue {
            session,
            name: None,
            data: session.to_string().into_bytes(),
            flag: FIFO,
            time: Timestamp::default(),
        };
        assert_eq!(status(&mut core, client(session), request), OK);
    }
    let query = |session| Request::QueryQueue {
        session,
        name: name("SESSION"),
    };
    assert_eq!(
        reply(&mut core, client(1), query(7)),
        Reply::QueueCount {
            status: OK,
            count: Some(0)
        },
        "a handle never seen is an empty queue"
    );
    assert_eq!(
        reply(&mut core, client(3), query(1)),
        Reply::QueueCount {
            status: OK,
            count: Some(1)
        },
        "a child with the handle shares the queue"
    );
    assert!(
        core.is_empty(),
        "a session queue holding lines is not held state"
    );
}

#[test]
fn a_session_wait_works_and_a_child_satisfies_it() {
    let mut core = Core::new();
    assert_eq!(core.handle(client(1), pull(None, WAIT)), Outcome::Parked);
    assert_eq!(status(&mut core, client(2), add(None, "line", FIFO)), OK);
    let ready = core.take_ready();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].0, client(1));
}

/// Resolution 4: a session queue goes when no connection uses it, whatever it holds.
#[test]
fn a_session_queue_goes_once_unused() {
    let mut core = Core::new();
    let attach = Request::CreateSessionQueue { session: SESSION };
    assert_eq!(status(&mut core, client(1), attach.clone()), OK);
    assert_eq!(status(&mut core, client(1), attach), OK, "repeatable");
    let detach = Request::DeleteSessionQueue { session: SESSION };
    assert_eq!(status(&mut core, client(1), detach.clone()), OK);
    assert_eq!(status(&mut core, client(1), detach), OK, "repeatable");
    core.client_closed(client(1));
    assert!(core.is_empty(), "an empty unused session queue is gone");

    assert_eq!(status(&mut core, client(2), add(None, "a", FIFO)), OK);
    core.client_closed(client(2));
    assert_eq!(
        count(&mut core, None),
        Reply::QueueCount {
            status: OK,
            count: Some(0)
        },
        "lines go with the last user"
    );
    core.client_closed(client(1));

    let parent = Request::CreateSessionQueue { session: SESSION };
    assert_eq!(status(&mut core, client(3), parent), OK);
    assert_eq!(status(&mut core, client(4), add(None, "b", FIFO)), OK);
    core.client_closed(client(4));
    assert_eq!(
        reply(&mut core, client(3), pull(None, NO_WAIT)),
        Reply::Pulled {
            status: OK,
            item: Some(Item {
                data: b"b".to_vec(),
                time: Timestamp::default(),
            }),
        },
        "a writer's lines stay while another connection uses the queue"
    );
    core.client_closed(client(3));
    assert!(core.is_empty());
}

const LIB: &[u8] = b"lib";

fn register(name: &str, library: &[u8], owner_only: bool) -> Request {
    Request::Register {
        registry: Registry::Subcom,
        name: name.as_bytes().to_vec(),
        library: library.to_vec(),
        procedure: b"proc".to_vec(),
        user_area: None,
        owner_only,
    }
}

fn query(name: &str, library: &[u8]) -> Request {
    Request::QueryRegistration {
        registry: Registry::Subcom,
        name: name.as_bytes().to_vec(),
        library: library.to_vec(),
    }
}

fn drop_reg(name: &str, library: &[u8]) -> Request {
    Request::Deregister {
        registry: Registry::Subcom,
        name: name.as_bytes().to_vec(),
        library: library.to_vec(),
    }
}

#[test]
fn registering_pairs_and_duplicates() {
    let mut core = Core::new();
    let area = *b"0123456789abcdef";
    let first = Request::Register {
        registry: Registry::Subcom,
        name: b"Env".to_vec(),
        library: b"Lib".to_vec(),
        procedure: b"First".to_vec(),
        user_area: Some(area),
        owner_only: false,
    };
    assert_eq!(status(&mut core, client(1), first), OK);
    let duplicate = Request::Register {
        registry: Registry::Subcom,
        name: b"ENV".to_vec(),
        library: b"LIB".to_vec(),
        procedure: b"Second".to_vec(),
        user_area: None,
        owner_only: true,
    };
    assert_eq!(status(&mut core, client(2), duplicate), DUPLICATE);
    assert_eq!(
        reply(&mut core, client(1), query("env", b"lib")),
        Reply::Registration {
            status: OK,
            registered: Some(Registered {
                library: b"Lib".to_vec(),
                procedure: b"First".to_vec(),
                user_area: area,
            })
        },
        "a duplicate leaves everything but the references"
    );
    assert_eq!(
        status(&mut core, client(1), register("env", b"other", false)),
        OK,
        "a different library is a new pair"
    );
    assert_eq!(
        reply(&mut core, client(1), query("missing", b"")),
        Reply::Registration {
            status: NOT_REGISTERED,
            registered: None
        }
    );
    assert_eq!(
        status(&mut core, client(1), query("env", b"third")),
        NOT_REGISTERED
    );
}

#[test]
fn user_area_defaults_to_zeros_and_empty_library_is_name_only() {
    let mut core = Core::new();
    status(&mut core, client(1), register("env", b"", false));
    assert_eq!(
        reply(&mut core, client(1), query("env", b"")),
        Reply::Registration {
            status: OK,
            registered: Some(Registered {
                library: Vec::new(),
                procedure: b"proc".to_vec(),
                user_area: [0; 16],
            })
        }
    );
}

#[test]
fn the_three_registries_are_independent() {
    let mut core = Core::new();
    for registry in [Registry::Function, Registry::Subcom, Registry::Exit] {
        let request = Request::Register {
            registry,
            name: b"name".to_vec(),
            library: LIB.to_vec(),
            procedure: b"proc".to_vec(),
            user_area: None,
            owner_only: false,
        };
        assert_eq!(status(&mut core, client(1), request), OK);
    }
    assert_eq!(status(&mut core, client(1), drop_reg("name", b"")), OK);
    assert_eq!(
        status(&mut core, client(1), query("name", b"")),
        NOT_REGISTERED
    );
    let exit = Request::QueryRegistration {
        registry: Registry::Exit,
        name: b"name".to_vec(),
        library: Vec::new(),
    };
    assert_eq!(status(&mut core, client(1), exit), OK);
}

#[test]
fn names_truncate_to_255_bytes_and_any_string_works() {
    let mut core = Core::new();
    let long = "n".repeat(300);
    assert_eq!(
        status(&mut core, client(1), register(&long, LIB, false)),
        OK
    );
    assert_eq!(status(&mut core, client(1), query(&long[..255], LIB)), OK);
    assert_eq!(status(&mut core, client(1), query(&long, LIB)), OK);
    assert_eq!(
        status(&mut core, client(1), query(&long[..254], LIB)),
        NOT_REGISTERED
    );
    assert_eq!(status(&mut core, client(1), register("", LIB, false)), OK);
    assert_eq!(status(&mut core, client(1), query("", b"")), OK);
}

#[test]
fn name_only_operations_act_on_the_most_recently_touched() {
    let mut core = Core::new();
    let library_of = |core: &mut Core| match reply(core, client(1), query("env", b"")) {
        Reply::Registration {
            registered: Some(registered),
            ..
        } => registered.library,
        other => panic!("{other:?}"),
    };
    status(&mut core, client(1), register("env", b"one", false));
    status(&mut core, client(1), register("env", b"two", false));
    assert_eq!(library_of(&mut core), b"two");
    assert_eq!(status(&mut core, client(1), query("env", b"one")), OK);
    assert_eq!(
        library_of(&mut core),
        b"one",
        "a query with its library touches"
    );
    assert_eq!(
        status(&mut core, client(1), register("env", b"two", false)),
        DUPLICATE
    );
    assert_eq!(library_of(&mut core), b"two", "a register touches");
    assert_eq!(status(&mut core, client(1), drop_reg("env", b"")), OK);
    assert_eq!(
        status(&mut core, client(1), query("env", b"two")),
        OK,
        "the drop took a duplicate's reference"
    );
    assert_eq!(status(&mut core, client(1), drop_reg("env", b"one")), OK);
    assert_eq!(library_of(&mut core), b"two");
}

#[test]
fn register_twice_drops_as_test_register_twice_asserts() {
    let mut core = Core::new();
    assert_eq!(
        status(&mut core, client(1), register("env", LIB, false)),
        OK
    );
    assert_eq!(
        status(&mut core, client(1), register("env", LIB, false)),
        DUPLICATE
    );
    assert_eq!(status(&mut core, client(1), drop_reg("env", b"")), OK);
    assert_eq!(status(&mut core, client(1), drop_reg("env", LIB)), OK);
    assert_eq!(
        status(&mut core, client(1), drop_reg("env", b"")),
        NOT_REGISTERED
    );
}

#[test]
fn a_drop_with_library_leaving_references_answers_10() {
    let mut core = Core::new();
    status(&mut core, client(1), register("env", LIB, false));
    status(&mut core, client(2), register("env", LIB, false));
    assert_eq!(
        status(&mut core, client(3), drop_reg("env", LIB)),
        DUPLICATE
    );
    assert_eq!(status(&mut core, client(3), drop_reg("env", LIB)), OK);
    assert_eq!(
        status(&mut core, client(3), drop_reg("env", LIB)),
        NOT_REGISTERED
    );
}

#[test]
fn owner_only_drops_by_owner_and_outlives_it() {
    let mut core = Core::new();
    assert_eq!(status(&mut core, client(1), register("env", LIB, true)), OK);
    assert_eq!(
        status(&mut core, client(2), drop_reg("env", LIB)),
        NO_CAN_DROP
    );
    assert_eq!(
        status(&mut core, client(2), drop_reg("env", b"")),
        NO_CAN_DROP
    );
    assert_eq!(status(&mut core, client(2), query("env", b"")), OK);

    core.client_closed(client(1));
    assert!(!core.is_empty(), "a registration outlives its registrant");
    assert_eq!(
        status(&mut core, client(9), drop_reg("env", b"")),
        NO_CAN_DROP,
        "a later connection is never the owner",
    );
}

#[test]
fn a_call_on_the_owners_connection_is_the_owner() {
    let mut core = Core::new();
    status(&mut core, client(1).call(1), register("env", LIB, true));
    assert_eq!(
        status(&mut core, client(1).call(2), drop_reg("env", b"")),
        OK
    );
}

#[test]
fn an_owner_whose_reference_is_gone_repeats_the_drop_forever() {
    let mut core = Core::new();
    status(&mut core, client(1), register("env", LIB, true));
    assert_eq!(
        status(&mut core, client(2), register("env", LIB, true)),
        DUPLICATE
    );
    assert_eq!(status(&mut core, client(1), drop_reg("env", b"")), OK);
    assert_eq!(status(&mut core, client(1), drop_reg("env", b"")), OK);
    assert_eq!(
        status(&mut core, client(1), drop_reg("env", LIB)),
        DUPLICATE
    );
    assert_eq!(status(&mut core, client(2), query("env", LIB)), OK);
    assert_eq!(
        status(&mut core, client(2), drop_reg("env", b"")),
        NO_CAN_DROP
    );
}

#[test]
fn a_droppable_registration_with_an_owner_only_duplicate_cannot_be_removed() {
    let mut core = Core::new();
    status(&mut core, client(1), register("env", LIB, false));
    status(&mut core, client(2), register("env", LIB, true));
    assert_eq!(status(&mut core, client(1), drop_reg("env", b"")), OK);
    assert_eq!(status(&mut core, client(1), drop_reg("env", b"")), OK);
    assert_eq!(status(&mut core, client(2), drop_reg("env", b"")), OK);
    assert_eq!(status(&mut core, client(3), query("env", b"")), OK);
}

#[test]
fn an_owner_only_reference_counts_against_its_own_process() {
    let mut core = Core::new();
    status(&mut core, client(1), register("env", LIB, true));
    assert_eq!(
        status(&mut core, client(1), register("env", LIB, true)),
        DUPLICATE
    );
    assert_eq!(
        status(&mut core, client(1), drop_reg("env", LIB)),
        DUPLICATE
    );
    assert_eq!(status(&mut core, client(1), drop_reg("env", LIB)), OK);
    assert_eq!(
        status(&mut core, client(1), drop_reg("env", LIB)),
        NOT_REGISTERED
    );
}

fn function(name: &str, library: &str) -> Request {
    Request::Register {
        registry: Registry::Function,
        name: name.as_bytes().to_vec(),
        library: library.as_bytes().to_vec(),
        procedure: b"proc".to_vec(),
        user_area: None,
        owner_only: false,
    }
}

#[test]
fn function_registrations_are_always_owner_only() {
    let mut core = Core::new();
    assert_eq!(status(&mut core, client(1), function("F", "lib")), OK);
    assert_eq!(
        status(&mut core, client(1), function("f", "lib")),
        DUPLICATE
    );
    let deregister = Request::Deregister {
        registry: Registry::Function,
        name: b"F".to_vec(),
        library: Vec::new(),
    };
    assert_eq!(
        status(&mut core, client(2), deregister.clone()),
        NO_CAN_DROP
    );
    assert_eq!(status(&mut core, client(1), deregister.clone()), OK);
    assert_eq!(status(&mut core, client(1), deregister.clone()), OK);
    assert_eq!(status(&mut core, client(1), deregister), NOT_REGISTERED);
}

fn add_macro(name: &str, image: &str, position: u64) -> Request {
    Request::AddMacro {
        name: name.as_bytes().to_vec(),
        image: image.as_bytes().to_vec(),
        position,
    }
}

fn macro_named(name: &str) -> Vec<u8> {
    name.as_bytes().to_vec()
}

fn position(core: &mut Core, name: &str) -> Reply {
    reply(
        core,
        client(1),
        Request::QueryMacro {
            name: macro_named(name),
        },
    )
}

fn save(core: &mut Core, names: Option<&[&str]>) -> Reply {
    reply(
        core,
        client(1),
        Request::SaveMacros {
            names: names.map(|names| names.iter().map(|name| macro_named(name)).collect()),
        },
    )
}

#[test]
fn macros_add_replace_query_reorder_drop() {
    let mut core = Core::new();
    assert_eq!(status(&mut core, client(1), add_macro("Mac", "one", 1)), OK);
    assert_eq!(status(&mut core, client(1), add_macro("MAC", "two", 2)), OK);
    assert_eq!(
        position(&mut core, "mac"),
        Reply::MacroPosition {
            status: OK,
            position: 2
        },
        "an existing macro is replaced, with its position"
    );
    assert_eq!(
        reply(
            &mut core,
            client(1),
            Request::ResolveMacro {
                name: macro_named("mAc")
            }
        ),
        Reply::MacroImage {
            status: OK,
            image: Some(b"two".to_vec())
        }
    );
    for stored in [0, 3, 99, 70000] {
        let reorder = Request::ReorderMacro {
            name: macro_named("mac"),
            position: stored,
        };
        assert_eq!(status(&mut core, client(1), reorder), OK);
        assert_eq!(
            position(&mut core, "mac"),
            Reply::MacroPosition {
                status: OK,
                position: stored
            }
        );
    }
    let drop = Request::DropMacro {
        name: macro_named("MAC"),
    };
    assert_eq!(status(&mut core, client(1), drop.clone()), OK);
    assert_eq!(status(&mut core, client(1), drop), MACRO_NOT_FOUND);
    assert_eq!(
        position(&mut core, "mac"),
        Reply::MacroPosition {
            status: MACRO_NOT_FOUND,
            position: 0
        }
    );
    let reorder = Request::ReorderMacro {
        name: macro_named("mac"),
        position: 1,
    };
    assert_eq!(status(&mut core, client(1), reorder), MACRO_NOT_FOUND);
    assert_eq!(
        reply(
            &mut core,
            client(1),
            Request::ResolveMacro {
                name: macro_named("mac")
            }
        ),
        Reply::MacroImage {
            status: MACRO_NOT_FOUND,
            image: None
        }
    );
}

#[test]
fn macro_names_truncate_and_the_empty_name_works() {
    let mut core = Core::new();
    let long = "m".repeat(300);
    status(&mut core, client(1), add_macro(&long, "image", 1));
    assert!(matches!(
        position(&mut core, &long[..255]),
        Reply::MacroPosition { status: OK, .. }
    ));
    assert_eq!(status(&mut core, client(1), add_macro("", "image", 1)), OK);
    assert!(matches!(
        position(&mut core, ""),
        Reply::MacroPosition { status: OK, .. }
    ));
    assert_eq!(status(&mut core, client(1), Request::ClearMacroSpace), OK);
    assert_eq!(status(&mut core, client(1), Request::ClearMacroSpace), OK);
    assert!(core.is_empty());
}

#[test]
fn save_lists_macros_or_refuses() {
    let mut core = Core::new();
    assert_eq!(
        save(&mut core, None),
        Reply::Macros {
            status: MACRO_FILE_ERROR,
            macros: Vec::new()
        },
        "saving an empty macrospace"
    );
    assert_eq!(
        save(&mut core, Some(&[])),
        Reply::Macros {
            status: OK,
            macros: Vec::new()
        },
        "count 0 with names"
    );
    status(&mut core, client(1), add_macro("a", "1", 1));
    status(&mut core, client(1), add_macro("b", "2", 2));
    let macro_a = Macro {
        name: b"a".to_vec(),
        image: b"1".to_vec(),
        position: 1,
    };
    let macro_b = Macro {
        name: b"b".to_vec(),
        image: b"2".to_vec(),
        position: 2,
    };
    assert_eq!(
        save(&mut core, Some(&["B", "A"])),
        Reply::Macros {
            status: OK,
            macros: vec![macro_b.clone(), macro_a.clone()]
        },
        "in the order listed"
    );
    assert_eq!(
        save(&mut core, Some(&["a", "missing"])),
        Reply::Macros {
            status: MACRO_NOT_FOUND,
            macros: Vec::new()
        }
    );
    let Reply::Macros {
        status: OK,
        mut macros,
    } = save(&mut core, None)
    else {
        panic!("save everything");
    };
    macros.sort_by(|left, right| left.name.cmp(&right.name));
    assert_eq!(macros, [macro_a, macro_b]);
}

#[test]
fn load_replaces_keeps_and_filters_by_name() {
    let mut core = Core::new();
    status(&mut core, client(1), add_macro("kept", "old", 1));
    status(&mut core, client(1), add_macro("same", "old", 1));
    let file = vec![
        Macro {
            name: b"SAME".to_vec(),
            image: b"new".to_vec(),
            position: 2,
        },
        Macro {
            name: b"fresh".to_vec(),
            image: b"new".to_vec(),
            position: 2,
        },
        Macro {
            name: b"skipped".to_vec(),
            image: b"new".to_vec(),
            position: 2,
        },
    ];
    let load = |names: Option<Vec<Vec<u8>>>| Request::LoadMacros {
        names,
        macros: file.clone(),
    };
    assert_eq!(
        status(
            &mut core,
            client(1),
            load(Some(vec![macro_named("Fresh"), macro_named("not in file")]))
        ),
        OK
    );
    assert!(matches!(
        position(&mut core, "fresh"),
        Reply::MacroPosition { status: OK, .. }
    ));
    assert!(
        matches!(
            position(&mut core, "same"),
            Reply::MacroPosition {
                status: OK,
                position: 1
            }
        ),
        "an unlisted macro is not loaded"
    );
    assert_eq!(status(&mut core, client(1), load(None)), OK);
    assert!(matches!(
        position(&mut core, "same"),
        Reply::MacroPosition {
            status: OK,
            position: 2
        }
    ));
    assert!(matches!(
        position(&mut core, "kept"),
        Reply::MacroPosition {
            status: OK,
            position: 1
        }
    ));
}

#[test]
fn is_empty_counts_named_queues_registrations_and_macros() {
    let mut core = Core::new();
    assert!(core.is_empty());

    open(&mut core, "q");
    assert!(!core.is_empty());
    delete(&mut core, Some("q"));
    assert!(core.is_empty());

    status(&mut core, client(1), register("env", LIB, false));
    assert!(!core.is_empty());
    status(&mut core, client(1), drop_reg("env", b""));
    assert!(core.is_empty());

    status(&mut core, client(1), add_macro("m", "image", 1));
    assert!(!core.is_empty());
    status(
        &mut core,
        client(1),
        Request::DropMacro {
            name: macro_named("m"),
        },
    );
    assert!(core.is_empty());
}
