use super::run_program;

/// A chain of `Stem` subclass instances, each the next one's default, is
/// converted and rendered in one walk, and a message it forwards walks it
/// once: 50000 links end well inside the thread's timeout, where a walk per
/// link would not.
#[test]
fn a_long_chain_of_stem_subclass_instances_is_walked_once() {
    let source = b"first = .stem~subclass('m')~new\nt = first\ndo 49999\n  u = .stem~subclass('m')~new\n  t[] = u\n  t = u\nend\nt[] = 'leaf'\n\
                   say length(first) first~string first\nsay (first || 'x') (first = 'leaf') first~length\n\
                   trace r\nx = first\ntrace o\nsay 'end'\n"
        .to_vec();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let outcome = run_program("/w/l.rex", source, crate::Invocation::none());
        let _ = sender.send(outcome);
    });
    let outcome = receiver
        .recv_timeout(std::time::Duration::from_secs(20))
        .unwrap_or_else(|_| panic!("the chain did not end"));
    assert_eq!(
        outcome.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "4 leaf leaf\nleafx 1 4\nend\n"
    );
    assert!(
        String::from_utf8_lossy(&outcome.stderr).contains(">>>   \"leaf\""),
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
}

/// A `DEFAULTNAME` that renders its receiver inside an Array join recurses
/// until the control stack is full, and that condition is the join's: 11.1,
/// rc 245, as on the oracle (t87c-review-scratch/c1r/r4.rex).
#[test]
fn a_join_rendering_itself_through_defaultname_runs_out_of_stack() {
    let outcome = run_program(
        "/w/r.rex",
        b"o = .rec~new\nsay .array~of(o)~makestring\nsay 'after'\nexit\n\
          ::class rec\n::method defaultname\n  return .array~of(self)~makestring\n"
            .to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 245);
    assert!(outcome.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&outcome.stderr).ends_with(
            "Error 11.1:  Insufficient control stack space; cannot continue execution.\n"
        ),
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
}
