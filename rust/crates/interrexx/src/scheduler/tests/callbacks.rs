//! API callbacks from native calls on the interpreter's pool threads and
//! from threads running none, over routines
//! this test and `interrexx_api::load` define.

use std::sync::Mutex;
use std::thread::ThreadId;
use std::time::Duration;

use interrexx_api::layout::{RexxCallContext_, ValueDescriptor};
use interrexx_api::values::{ARGUMENT_TERMINATOR, code};

use super::pool::{Ran, SHAPE, Shape, run_shaped};

/// A routine that answers nothing.
static SIGNATURE: [u16; 2] = [code::REXX_OBJECT_PTR, ARGUMENT_TERMINATOR];

/// The threads [`here`] ran on, each under the test's number.
static THREADS: Mutex<Vec<(usize, ThreadId)>> = Mutex::new(Vec::new());

extern "C-unwind" fn here<const TEST: usize>(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return SIGNATURE.as_ptr().cast_mut();
    }
    THREADS
        .lock()
        .expect("unpoisoned")
        .push((TEST, std::thread::current().id()));
    std::ptr::null_mut()
}

/// Sleeps `MILLIS` milliseconds.
extern "C-unwind" fn nap<const MILLIS: u64>(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return SIGNATURE.as_ptr().cast_mut();
    }
    std::thread::sleep(Duration::from_millis(MILLIS));
    std::ptr::null_mut()
}

fn library() -> interrexx_api::load::Library {
    interrexx_api::load::routines_only(&[
        ("SENDTWICE", interrexx_api::load::send_twice),
        ("SENDAWAITSEND", interrexx_api::load::send_await_send),
        ("SENDKEEPING", interrexx_api::load::send_keeping),
        ("NAPTHENSEND", interrexx_api::load::nap_then_send),
        ("AWAITSENDTWICE", interrexx_api::load::await_send_twice),
        ("AWAITTHENSEND", interrexx_api::load::await_then_send),
        ("SIGNALAWAITSEND", interrexx_api::load::signal_await_send),
        ("SENDTHROUGH", interrexx_api::load::send_through),
        ("KEEPCALLCONTEXT", interrexx_api::load::keep_call_context),
        ("SETTHROUGH", interrexx_api::load::set_through),
        ("NAPTHENSET", interrexx_api::load::nap_then_set),
        (
            "FOREIGNINCALLBACK",
            interrexx_api::load::foreign_in_callback,
        ),
        ("FOREIGNTHEN", interrexx_api::load::foreign_then),
        (
            "SENDFROMANOTHERTHREAD",
            interrexx_api::load::send_from_another_thread,
        ),
        ("HERE1", here::<1>),
        ("HERE2", here::<2>),
        ("HERE3", here::<3>),
        ("NAPLONG", nap::<1500>),
        ("NAP", nap::<300>),
    ])
}

/// Runs `source` with this module's routines as the library `callbacktest`.
fn run(source: &str) -> Ran {
    run_shaped(
        source,
        Shape {
            library: (b"callbacktest", library),
            ..SHAPE
        },
    )
    .expect("the run did not panic")
}

/// The threads `HERE` ran on for the test numbered `test`.
fn threads_of(test: usize) -> Vec<ThreadId> {
    THREADS
        .lock()
        .expect("unpoisoned")
        .iter()
        .filter(|(ran_for, _)| *ran_for == test)
        .map(|(_, thread)| *thread)
        .collect()
}

/// A native call on a pool thread calls back from its own thread: each
/// callback runs, on that thread, and the call answers what the last one
/// answered.
#[test]
fn a_callback_from_the_calls_own_thread_runs() {
    let ran = run(
        "m = .t~new~start('idle')\nsay SENDTWICE(.r~new, 'A', 'B')\nsay m~result\n\
         ::requires 'callbacktest' LIBRARY\n::class t\n::method idle\n  call SysSleep 0.2\n  \
         return 'idle'\n::class r\n::method a\n  call HERE1\n  return 'a'\n\
         ::method b\n  return 'b'\n",
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "b\nidle\n");
    assert_eq!(ran.exits, 1);
    let threads = threads_of(1);
    assert_eq!(threads.len(), 1);
    assert_ne!(
        threads[0], ran.thread,
        "the callback ran on the interpreter's thread"
    );
}

/// A callback from a thread running no native call takes the baton, does
/// nothing, and leaves the oracle's 98.983 on the call whose context it used.
/// The holder serves it while it runs another activity, which it switches out
/// for the call's own; the baton is held once, for the whole member.
// spec: native.threads.context.deviation.19
#[test]
fn a_callback_from_another_thread_takes_the_baton_and_raises_98_983() {
    let ran = run("m = .t~new~start('spin')\nsignal on syntax\n\
         say SENDFROMANOTHERTHREAD(.loud~new, 'SPEAK')\nsay 'unreached'\n\
         syntax:\nsay 'trapped' condition('o')~code\nsay m~result\n\
         ::requires 'callbacktest' LIBRARY\n::class t\n::method spin\n  \
         do i = 1 to 2000000\n  end\n  return 'spun'\n\
         ::class loud\n::method speak\n  say 'spoke'\n  return 1\n");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "trapped 98.983\nspun\n");
    assert_eq!(ran.exits, 1);
    assert_eq!(ran.takes, 1, "the other thread took the baton once");
}

/// In the simulation mode the same callback, made while the call runs on
/// the baton, is refused at once: the call's own thread does nothing, the
/// call returns, and the run is refused at the next clause boundary rather
/// than waiting on a thread whose timing is real.
#[test]
fn a_callback_from_another_thread_is_refused_in_the_simulation_mode() {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let ran = run_shaped(
            "signal on syntax\nsay SENDFROMANOTHERTHREAD(.loud~new, 'SPEAK')\nsay 'after'\n\
             exit\nsyntax:\nsay 'trapped' condition('o')~code\n\
             ::requires 'callbacktest' LIBRARY\n::class loud\n::method speak\n  say 'spoke'\n  \
             return 1\n",
            Shape {
                library: (b"callbacktest", library),
                switch: Some(crate::SwitchMode::Sim(
                    crate::SimConfig::parse("sim:1").expect("a sim spec"),
                )),
                ..SHAPE
            },
        )
        .expect("the run did not panic");
        let _ = sender.send(ran);
    });
    let ran = receiver
        .recv_timeout(Duration::from_secs(20))
        .expect("the run ended");
    assert_eq!(ran.outcome.exit_code, 120);
    assert_eq!(ran.stdout(), "1\n");
    assert_eq!(
        ran.stderr(),
        "interrexx: a callback from another thread in the simulation mode is not implemented\n"
    );
}

/// A thread context kept from a call on one pool thread, used from another
/// activity's call on another pool thread while the first call runs outside
/// any callback, does nothing and gives the user's own call 98.983.
#[test]
fn a_kept_context_used_from_another_threads_call_raises_98_983_there() {
    let ran = run("k = .keeper~new\nu = .t~new~start('use', k)\n\
         call SENDKEEPING k, 'KEEP'\nsay u~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class keeper\n::method init\n  expose kept\n  kept = .nil\n\
         ::method keep unguarded\n  expose kept\n  use arg kept\n\
         ::method kept unguarded\n  expose kept\n  return kept\n\
         ::class t\n::method use\n  use arg k\n  do while k~kept == .nil\n    call SysSleep 0.01\n  end\n  \
         signal on syntax\n  call SENDTHROUGH k~kept, .loud~new, 'SPEAK'\n  return 'answered'\n\
         syntax:\n  return 'trapped' condition('o')~code\n\
         ::class loud\n::method speak\n  say 'spoke'\n  return 1\n");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "trapped 98.983\n");
}

/// A call context kept from main's call on one pool thread, used from
/// another activity's call on another pool thread, sets nothing and gives
/// the user's own call 98.983.
#[test]
fn a_kept_call_context_used_from_another_threads_call_raises_98_983_there() {
    let ran = run(
        "k = .keeper~new\nu = .t~new~start('use', k)\nx = 'before'\n\
         call KEEPCALLCONTEXT k, 'KEEP'\nsay u~result x\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class keeper\n::method init\n  expose kept\n  kept = .nil\n\
         ::method keep unguarded\n  expose kept\n  use arg kept\n\
         ::method kept unguarded\n  expose kept\n  return kept\n\
         ::class t\n::method use\n  use arg k\n  do while k~kept == .nil\n    call SysSleep 0.01\n  end\n  \
         signal on syntax\n  call SETTHROUGH k~kept, 'X', 'set'\n  return 'answered'\n\
         syntax:\n  return 'trapped' condition('o')~code\n",
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "trapped 98.983 before\n");
}

/// A callback from a thread running no native call, served while the
/// call's own thread is inside a callback that waits, does nothing, and
/// leaves 98.983 on the call for when it returns.
#[test]
fn a_foreign_callback_while_the_calls_thread_is_in_a_callback_raises_98_983() {
    let ran = run("m = .t~new~start('spin')\nsignal on syntax\n\
         call FOREIGNINCALLBACK .w~new, 'WAIT', .loud~new, 'SPEAK'\nsay 'unreached'\n\
         syntax:\nsay 'trapped' condition('o')~code\nsay m~result\n\
         ::requires 'callbacktest' LIBRARY\n::class t\n::method spin\n  \
         do i = 1 to 2000000\n  end\n  return 'spun'\n\
         ::class w\n::method wait\n  call SysSleep 0.5\n  return 1\n\
         ::class loud\n::method speak\n  say 'spoke'\n  return 1\n");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "trapped 98.983\nspun\n");
}

/// The 98.983 a foreign callback leaves on the call is sticky (queue item 2): the call's own
/// `ClearCondition` afterwards does not drop it, and its own raise afterwards does not replace
/// it.
#[test]
fn a_foreign_callbacks_98_983_survives_a_later_clear_or_raise() {
    for after in ["CLEAR", "RAISE"] {
        let ran = run(&format!(
            "m = .t~new~start('spin')\nsignal on syntax\n\
             call FOREIGNTHEN .w~new, 'WAIT', .loud~new, 'SPEAK', '{after}'\nsay 'unreached'\n\
             syntax:\nsay 'trapped' condition('o')~code\nsay m~result\n\
             ::requires 'callbacktest' LIBRARY\n::class t\n::method spin\n  \
             do i = 1 to 2000000\n  end\n  return 'spun'\n\
             ::class w\n::method wait\n  call SysSleep 0.5\n  return 1\n\
             ::class loud\n::method speak\n  say 'spoke'\n  return 1\n"
        ));
        assert_eq!(ran.outcome.exit_code, 0, "{after}: {}", ran.stderr());
        assert_eq!(ran.stdout(), "trapped 98.983\nspun\n", "{after}");
    }
}

/// A callback whose call's activity a pinned waiter waits for is answered by
/// the waiter's nested loop: the waiter's thread lends the baton to the
/// call's thread, which runs the callback.
#[test]
fn a_callback_while_the_holder_is_pinned_is_answered() {
    let ran = run(
        "m = .t~new~start('work')\na = .array~of(2, 1)\na~sortWith(.c~new(m))\n\
         say 'sorted' a~toString('L', ',')\n\
         ::requires 'callbacktest' LIBRARY\n::class t\n::method work\n  \
         return SENDTWICE(.r~new, 'A', 'B')\n\
         ::class r\n::method a\n  call HERE2\n  return 'a'\n::method b\n  return 'b'\n\
         ::class c\n::method init\n  expose m\n  use arg m\n\
         ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  return l - r\n",
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "b\nsorted 1,2\n");
    assert_eq!(ran.exits, 1);
    let threads = threads_of(2);
    assert_eq!(threads.len(), 1);
    assert_ne!(
        threads[0], ran.thread,
        "the callback ran on the interpreter's thread"
    );
}

/// Two activities in native calls on pool threads call back in turn. The
/// first callback starts the second activity just before parking in
/// `GUARD ON WHEN`, so the second callback can satisfy the pinned wait.
/// The second native call calls back again once the first has returned.
#[test]
fn two_activities_in_native_calls_with_interleaved_callbacks() {
    let path = std::env::temp_dir().join(format!("rexx-interleaved-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let ran = run(&format!(
        "m = .meet~new\nb = .message~new(.t~new, 'SECOND', 'I', .side~new(m, 'B'))\n\
         m~second = b\na = .t~new~start('first', .side~new(m, 'A'))\nsay a~result b~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method first\n  use arg side\n  r = SENDTWICE(side, 'FIRST', 'SECOND')\n  \
         call lineout '{path}', r\n  call lineout '{path}'\n  return r\n\
         ::method second\n  use arg side\n  return SENDAWAITSEND(side, 'FIRST', '{path}', 'SECOND')\n\
         ::class side\n::method init\n  expose m who\n  use arg m, who\n\
         ::method first\n  expose m who\n  call HERE3\n  m~arrive\n  return who'1'\n\
         ::method second\n  expose who\n  call HERE3\n  return who'2'\n\
         ::class meet\n::attribute second\n::method init\n  expose n\n  n = 0\n\
         ::method arrive\n  expose n second\n  n += 1\n  if n == 1 then second~start\n  \
         guard on when n >= 2\n",
        path = path.display()
    ));
    let written = std::fs::read_to_string(&path).unwrap_or_default();
    let _ = std::fs::remove_file(&path);
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "A2 B2\n");
    assert_eq!(written, "A2\n");
    let threads = threads_of(3);
    assert_eq!(threads.len(), 4);
    assert!(
        !threads.contains(&ran.thread),
        "a callback ran on the interpreter's thread"
    );
    assert_ne!(threads[0], threads[1]);
    assert_eq!(threads[0], threads[2]);
    assert_eq!(threads[1], threads[3]);
}

/// A recall that arrives while another call's callback runs, and that a
/// clause boundary of that callback serves, runs on top of it, and its own
/// callback's wait for the one it interrupted is refused as an inverted wait.
/// B waits in `AWAITTHENSEND` for the file that A's first callback writes
/// before it spins until B's recall is served on top of it, which is the only
/// way the spin ends. Whether such a wait should instead run the interrupted
/// activity, whose run goes on only once the recall's lend returns, is an
/// open scheduling question; until it is settled the wait is refused.
#[test]
fn a_wait_for_a_callback_a_boundary_recall_interrupted_is_refused_as_inverted() {
    let path = std::env::temp_dir().join(format!("rexx-boundary-recall-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let ran = run(&format!(
        "m = .meet~new\nb = .t~new~start('waits', .side~new(m, 'B'))\n\
         a = .t~new~start('sends', .side~new(m, 'A'))\nsay a~result b~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method sends\n  use arg side\n  return SENDTWICE(side, 'FIRST', 'SECOND')\n\
         ::method waits\n  use arg side\n  return AWAITTHENSEND('{path}', side, 'FIRST')\n\
         ::class side\n::method init\n  expose m who\n  use arg m, who\n\
         ::method first\n  expose m who\n  if who == 'A' then do\n    \
         call lineout '{path}', 'go'\n    call lineout '{path}'\n    do forever\n    end\n  \
         end\n  m~arrive\n  return who'1'\n\
         ::method second\n  expose who\n  return who'2'\n\
         ::class meet\n::method init\n  expose n\n  n = 0\n\
         ::method arrive\n  expose n\n  n += 1\n  guard on when n >= 2\n",
        path = path.display()
    ));
    let _ = std::fs::remove_file(&path);
    assert_eq!(ran.outcome.exit_code, 120, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "");
    assert_eq!(
        ran.stderr(),
        "interrexx: a pinned wait for a guard's WHEN that only an activity pinned below it \
         can end is not implemented\n"
    );
}

/// A recall served while the activity that ran last is parked in its own
/// wait leaves that activity parked, not runnable, below the callback: a wait
/// of the callback's that nothing can end is refused as such, not as an
/// inverted wait. B's `SIGNALAWAITSEND` writes one file once it is in flight
/// and sends only once the other exists, which main writes in the clause that
/// parks it in its wait for B's result, so no clause boundary can serve B's
/// recall before main parks. B's callback waits for a message nothing sends.
/// The text is the same with or without the inverted-wait rule for
/// interrupted activities: the test guards that a recall a loop serves marks
/// the activity below it buried, not interrupted.
#[test]
fn a_wait_above_a_recall_served_while_the_last_activity_parks_is_refused_as_unending() {
    let dir = std::env::temp_dir();
    let id = std::process::id();
    let entered = dir.join(format!("rexx-parked-recall-entered-{id}"));
    let signal = dir.join(format!("rexx-parked-recall-{id}"));
    let _ = std::fs::remove_file(&entered);
    let _ = std::fs::remove_file(&signal);
    let ran = run(&format!(
        "b = .t~new~start('waits', .side~new)\n\
         do while stream('{entered}', 'c', 'query exists') == ''\n  call SysSleep 0.01\nend\n\
         say lineout('{signal}', 'go') b~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method waits\n  use arg side\n  \
         return SIGNALAWAITSEND('{entered}', '{signal}', side, 'FIRST')\n\
         ::class side\n::method first\n  return .message~new('abc', 'length')~result\n",
        entered = entered.display(),
        signal = signal.display()
    ));
    let _ = std::fs::remove_file(&entered);
    let _ = std::fs::remove_file(&signal);
    assert_eq!(ran.outcome.exit_code, 120, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "");
    assert_eq!(
        ran.stderr(),
        "interrexx: a wait that nothing left to run can end is not implemented\n"
    );
}

/// A wait above a recall served at a clause boundary of an activity that is
/// mid-run is refused as an inverted wait even where nothing, the
/// interrupted activity included, could end it: the text is a best guess
/// that does not ask what the wait is for. B's `SIGNALAWAITSEND` writes one
/// file once it is in flight and sends only once the other exists, which
/// main writes before it spins without parking, and stream output runs on
/// the baton, so main never parks once B can send: only a clause boundary of
/// main's spin can serve B's recall. B's callback waits for a message
/// nothing sends.
#[test]
fn a_wait_nothing_can_end_above_a_boundary_recall_is_refused_as_inverted() {
    let dir = std::env::temp_dir();
    let id = std::process::id();
    let entered = dir.join(format!("rexx-spin-recall-entered-{id}"));
    let signal = dir.join(format!("rexx-spin-recall-{id}"));
    let _ = std::fs::remove_file(&entered);
    let _ = std::fs::remove_file(&signal);
    let ran = run(&format!(
        "b = .t~new~start('waits', .side~new)\n\
         do while stream('{entered}', 'c', 'query exists') == ''\n  call SysSleep 0.01\nend\n\
         call lineout '{signal}', 'go'\ndo forever\nend\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method waits\n  use arg side\n  \
         return SIGNALAWAITSEND('{entered}', '{signal}', side, 'FIRST')\n\
         ::class side\n::method first\n  return .message~new('abc', 'length')~result\n",
        entered = entered.display(),
        signal = signal.display()
    ));
    let _ = std::fs::remove_file(&entered);
    let _ = std::fs::remove_file(&signal);
    assert_eq!(ran.outcome.exit_code, 120, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "");
    assert_eq!(
        ran.stderr(),
        "interrexx: a pinned wait for a message's completion that only an activity pinned \
         below it can end is not implemented\n"
    );
}

/// A callback that sends to a guarded method another activity holds waits,
/// pinned, on the callback's thread, whose nested loop runs the holder until
/// it releases the guard.
#[test]
fn a_callback_waits_for_a_contended_guard_on_its_own_thread() {
    let ran = run("g = .g~new\nh = .t~new~start('hold', g)\n\
         do while g~holding == 0\n  call SysSleep 0.01\nend\n\
         say SENDTWICE(g, 'TAKE', 'TAKE')\nsay h~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method hold\n  use arg g\n  return g~hold\n\
         ::class g\n::method init\n  expose holding\n  holding = 0\n\
         ::method holding unguarded\n  expose holding\n  return holding\n\
         ::method hold\n  expose holding\n  holding = 1\n  call SysSleep 0.3\n  say 'releases'\n  \
         return 'held'\n::method take\n  say 'taken'\n  return 'took'\n");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "releases\ntaken\ntaken\ntook\nheld\n");
    assert_eq!(ran.exits, 1);
}

/// A callback is served at the holder's next cold visit even while the
/// holder is pinned and nothing else is ready, so neither a slice nor a
/// round of other activities ends its run: a pinned busy-wait sees what the
/// callback did.
#[test]
fn a_callback_is_served_at_a_pinned_holders_next_cold_visit() {
    let ran = run(
        "f = .flag~new\nm = .t~new~start('work', f)\na = .array~of(2, 1)\n\
         a~sortWith(.c~new(f))\nsay 'sorted' a~toString('L', ',') m~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method work\n  use arg f\n  return NAPTHENSEND(f, 'MARK')\n\
         ::class flag\n::method init\n  expose set\n  set = 0\n\
         ::method set unguarded\n  expose set\n  return set\n\
         ::method mark unguarded\n  expose set\n  set = 1\n  return 'marked'\n\
         ::class c\n::method init\n  expose f\n  use arg f\n\
         ::method compare\n  expose f\n  use arg l, r\n  do while f~set == 0\n  end\n  \
         return l - r\n",
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "sorted 1,2 marked\n");
}

/// Recalls that reach the holder together, while it keeps the baton for a
/// call at a pool that spawns no thread beyond its bound, are each served: the first callback waits for
/// the second, which its own thread's nested loop finds in the inbox rather
/// than behind the holder's lend.
#[test]
fn recalls_drained_together_are_each_served() {
    let path = std::env::temp_dir().join(format!("rexx-batched-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let ran = run_shaped(
        &format!(
            "m = .meet~new\na = .t~new~start('go', .side~new(m, 'A'))\n\
             b = .t~new~start('go', .side~new(m, 'B'))\ncall SysSleep 0.1\n\
             call lineout '{path}', 'go'\ncall lineout '{path}'\ncall NAP\n\
             say a~result b~result\n\
             ::requires 'callbacktest' LIBRARY\n\
             ::class t\n::method go\n  use arg side\n  \
             return AWAITSENDTWICE('{path}', side, 'FIRST', 'SECOND')\n\
             ::class side\n::method init\n  expose m who\n  use arg m, who\n\
             ::method first\n  expose m who\n  m~arrive\n  return who'1'\n\
             ::method second\n  expose who\n  return who'2'\n\
             ::class meet\n::method init\n  expose n\n  n = 0\n\
             ::method arrive\n  expose n\n  n += 1\n  guard on when n >= 2\n",
            path = path.display()
        ),
        Shape {
            library: (b"callbacktest", library),
            bound: Some(2),
            fixed: true,
            deadline: Duration::from_secs(20),
            ..SHAPE
        },
    )
    .expect("the run did not panic");
    let _ = std::fs::remove_file(&path);
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "A2 B2\n");
}

/// A call abandoned while it runs on a pool thread (its park answered with
/// Error 11.1 by the test's injection) completes no later call of its
/// activity: the second call returns only once its own native has.
#[test]
fn an_abandoned_calls_completion_does_not_complete_the_next_call() {
    let ran = run_shaped(
        "s = .t~new~start('idle')\nsignal on syntax name abandoned\n\
         call NAP\nsay 'unreached'\n\
         abandoned:\nsay 'abandoned' condition('o')~code\n\
         call time 'R'\ncall NAPLONG\nsay 'waited' (time('E') >= 1.2)\nsay s~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method idle\n  call SysSleep 2\n  return 'idle'\n",
        Shape {
            library: (b"callbacktest", library),
            fail_native_wait: true,
            ..SHAPE
        },
    )
    .expect("the run did not panic");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "abandoned 11.1\nwaited 1\nidle\n");
}

/// A callback from a call its activity abandoned while it ran reaches no
/// activity: it answers nothing, and the activity runs on.
#[test]
fn a_callback_from_an_abandoned_call_answers_nothing() {
    let ran = run_shaped(
        "s = .t~new~start('idle')\nsignal on syntax name abandoned\n\
         say NAPTHENSEND(.f~new, 'MARK')\nsay 'unreached'\n\
         abandoned:\nsay 'abandoned' condition('o')~code\n\
         call SysSleep 0.5\nsay 'after'\nsay s~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method idle\n  call SysSleep 1\n  return 'idle'\n\
         ::class f\n::method mark\n  say 'marked'\n  return 1\n",
        Shape {
            library: (b"callbacktest", library),
            fail_native_wait: true,
            ..SHAPE
        },
    )
    .expect("the run did not panic");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "abandoned 11.1\nafter\nidle\n");
}

/// A variable set through the call context of a call its activity
/// abandoned while it ran sets nothing.
#[test]
fn a_variable_set_from_an_abandoned_call_sets_nothing() {
    let ran = run_shaped(
        "s = .t~new~start('idle')\nx = 'before'\nsignal on syntax name abandoned\n\
         call NAPTHENSET 'X', 'set'\nsay 'unreached'\n\
         abandoned:\nsay 'abandoned' condition('o')~code\n\
         call SysSleep 0.5\nsay x\nsay s~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method idle\n  call SysSleep 1\n  return 'idle'\n",
        Shape {
            library: (b"callbacktest", library),
            fail_native_wait: true,
            ..SHAPE
        },
    )
    .expect("the run did not panic");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "abandoned 11.1\nbefore\nidle\n");
}
