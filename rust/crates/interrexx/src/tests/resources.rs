use super::run_program;
use interrexx_inventory::errors::expand;

/// A chain of `Stem` subclass instances, each the next one's default, is
/// converted and rendered in one walk, and a message it forwards walks it
/// once: 50000 links end well inside the thread's timeout, where a walk per
/// link would not.
#[test]
fn a_long_chain_of_stem_subclass_instances_is_walked_once() {
    // Forwarding recurses per link, and ThreadSanitizer's 65536-entry shadow stack has no end check.
    let links = if crate::test_process::thread_sanitizer() {
        5000
    } else {
        50000
    };
    let source = format!(
        "first = .stem~subclass('m')~new\nt = first\ndo {}\n  u = .stem~subclass('m')~new\n  t[] = u\n  t = u\nend\nt[] = 'leaf'\n\
         say length(first) first~string first\nsay (first || 'x') (first = 'leaf') first~length\n\
         trace r\nx = first\ntrace o\nsay 'end'\n",
        links - 1
    )
    .into_bytes();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let outcome = run_program("/w/l.rex", source, crate::Invocation::none());
        let _ = sender.send(outcome);
    });
    // Instrumentation gets a bounded wait without weakening the ordinary complexity guard.
    let timeout = if crate::test_process::thread_sanitizer() {
        180
    } else {
        20
    };
    let outcome = receiver
        .recv_timeout(std::time::Duration::from_secs(timeout))
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
    let instrumented = crate::test_process::thread_sanitizer();
    let run_program = |path, source, invocation| {
        if instrumented {
            // Leave 1 MiB above the unchanged 32 MiB control-stack reserve.
            crate::run_program_on_stack(path, source, invocation, 33 * 1024 * 1024)
        } else {
            run_program(path, source, invocation)
        }
    };
    if instrumented {
        // Prove rendering and continued execution fit before testing recursive exhaustion.
        let preflight = run_program(
            "/w/r.rex",
            b"o = .rec~new\nsay .array~of(o)~makestring\nsay 'after'\nexit\n\
              ::class rec\n::method defaultname\n  return 'rendered'\n"
                .to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(preflight.exit_code, 0);
        assert_eq!(preflight.stdout, b"rendered\nafter\n");
        assert!(preflight.stderr.is_empty());
    }
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
        String::from_utf8_lossy(&outcome.stderr).ends_with(&expand("Error 11.1:  ⟨11.1⟩\n")),
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
}

/// An `INTERPRET` that interprets itself nests fragments until the control
/// stack is full, and raises 11.1 a SYNTAX trap catches, as on the oracle,
/// rather than overflowing the interpreter thread's stack (#1102). No
/// expression level guards the stack on this path: `Op::ExecWith` hands the
/// instruction its value from a register.
///
/// **A regression aborts this unit-test binary rather than failing this
/// test**: without the guard the nesting overflows the thread's real stack,
/// and every test after it in the binary goes unreported. Run it on its own
/// (`cargo test -p interrexx --lib tests::resources::an_interpret`) when
/// checking that it fails with the guard reverted. The same holds for
/// [`the_deepest_interpret_traces_its_value_before_running_out_of_stack`].
#[test]
fn an_interpret_that_interprets_itself_runs_out_of_stack() {
    // 1 MiB above the 32 MiB control-stack reserve, so the limit is reached
    // in a few levels.
    let outcome = crate::run_program_on_stack(
        "/w/r.rex",
        b"signal on syntax\nzs = 'interpret zs'\ninterpret zs\n\
          syntax: say condition('o')~code\n"
            .to_vec(),
        crate::Invocation::none(),
        33 * 1024 * 1024,
    );
    assert_eq!(
        outcome.stdout,
        b"11.1\n",
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.exit_code, 0);
}

/// Under `TRACE R` the deepest `INTERPRET` still traces its `>>>` value line
/// before 11.1 is raised, as on the oracle: the stack check comes after the
/// value is traced and before the fragment's level is entered.
#[test]
fn the_deepest_interpret_traces_its_value_before_running_out_of_stack() {
    let outcome = crate::run_program_on_stack(
        "/w/r.rex",
        b"trace r\nsignal on syntax\nzs = 'interpret zs'\ninterpret zs\n\
          syntax: trace o; say condition('o')~code\n"
            .to_vec(),
        crate::Invocation::none(),
        33 * 1024 * 1024,
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(outcome.stdout, b"11.1\n", "{stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    let trap = lines
        .iter()
        .position(|line| *line == "     5 *-* syntax:")
        .unwrap_or_else(|| panic!("the trap's label is not traced: {stderr}"));
    assert!(trap >= 2, "{stderr}");
    assert_eq!(
        lines[trap - 2..trap],
        ["     4 *-* interpret zs", "       >>>   \"interpret zs\""],
        "{stderr}"
    );
    // The trap ends the run cleanly: no error report after it, and rc 0.
    assert_eq!(
        lines[trap..],
        ["     5 *-* syntax:", "     5 *-* trace o;"],
        "{stderr}"
    );
    assert_eq!(outcome.exit_code, 0, "{stderr}");
}
