#[test]
fn registered_executable_survives_dictionary_mutation_and_collection() {
    for mutation in [
        ".k~define('FOO', .k~method('OTHER'))",
        ".k~define('FOO')",
        ".k~define('FOO', .nil)",
    ] {
        for prefetch in [false, true] {
            let source = format!(
                "m=.k~method('BODY')~copy~~setPrivate~~setUnguarded\n.k~define('FOO',m)\n.local~installed=.k~method('FOO')\ndrop m\n.s~new~go\nexit\n::class k\n::method go\nself~foo\n::method body protected\n{}\n{mutation}\nm=.context~executable\nsay (m == .installed) m~isGuarded m~isPrivate m~isProtected\n::method other\nreturn 17\n::class s subclass k\n",
                if prefetch {
                    "before=.context~executable"
                } else {
                    "nop"
                }
            );
            let outcome = crate::run_program_collect_every_alloc(
                "/identity.rex",
                source.into_bytes(),
                crate::Invocation::none(),
            );
            assert_eq!(outcome.exit_code, 0, "{:?}", outcome);
            assert_eq!(
                outcome.stdout, b"1 0 1 1\n",
                "{mutation}, prefetch={prefetch}"
            );
        }
    }
}

#[test]
fn executing_copies_and_enhanced_metaclass_entries_survive_collection() {
    for (name, source, expected) in [
        (
            "identity",
            include_str!("../../../../../corpus/lang/executing_method_after_redefinition.rex"),
            "0 replace 1 0 1 1\nsame 1\n0 remove 1 0 1 1\nsame 1\n0 hide 1 0 1 1\nsame 1\n1 replace 1 0 1 1\nsame 1\n1 remove 1 0 1 1\nsame 1\n1 hide 1 0 1 1\nsame 1\nBAR 1 1 0 1\nFOO 1 0 1 1\n",
        ),
        (
            "enhanced",
            include_str!("../../../../../corpus/lang/enhanced_metaclass_method_identity.rex"),
            "entry pong 1\ninherited pong 1\nheld 1 1\nlisted 1 1\nfound 1\nheld 1 1\nlisted 1 1\nfound 1\nheld 1 1\nlisted 1 1\nfound 1\nobject 17 1\nordinary ordinary 1\n",
        ),
    ] {
        let outcome = crate::run_program_collect_every_alloc(
            name,
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 0, "{:?}", outcome);
        assert!(outcome.collections > 0, "collection must actually run");
        eprintln!("{name}: {} collections", outcome.collections);
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), expected, "{name}");
    }
}
thread_local! {
    static INLINED_METHODS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn count_inlined_method() {
    INLINED_METHODS.with(|count| count.set(count.get() + 1));
}

#[test]
fn quiet_leaf_methods_inline_and_keep_their_send_results() {
    let before = INLINED_METHODS.with(std::cell::Cell::get);
    let outcome = crate::execute(
        "/leaf.rex",
        br#"
o = .probe~new
do i = 1 to 1500
  saved = answer(o)
  result = 'stale'
  o~empty
  if symbol('result') = 'VAR' then exit 1
end
say saved
.probe~define('ANSWER', 'return 7')
say answer(o)
exit
answer: use arg receiver
  return receiver~answer
::class probe
::method answer
  return 'okay'
::method empty
"#
        .to_vec(),
        false,
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout, b"okay\n7\n");
    let inlined = INLINED_METHODS.with(std::cell::Cell::get) - before;
    assert!(inlined > 2900, "inlined {inlined} of 3000 leaf sends");
}

#[test]
fn leaf_methods_honor_the_callee_packages_trace() {
    let outcome = crate::execute(
        "/leaf-trace.rex",
        br#"
trace off
o = .probe~new
say o~answer
say o~answer
::options trace r
::class probe
::method answer
  return 'okay'
"#
        .to_vec(),
        false,
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout, b"okay\nokay\n");
    let trace = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(trace.matches(">I> Method \"ANSWER\"").count(), 2, "{trace}");
    assert_eq!(trace.matches("<I< Method \"ANSWER\"").count(), 2, "{trace}");
}

#[test]
fn an_inlined_literal_return_counts_as_one_clause() {
    let visits = |options| {
        let before = crate::clause::countdown_visits();
        let outcome = crate::execute(
            "/leaf-countdown.rex",
            format!(
                "o = .probe~new\n\
                 do 5000\n\
                   result = o~answer\n\
                 end\n\
                 say result\n\
                 ::class probe\n\
                 ::method answer\n\
                   return 'okay'\n\
                 {options}"
            )
            .into_bytes(),
            false,
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 0, "{outcome:?}");
        assert_eq!(outcome.stdout, b"okay\n");
        crate::clause::countdown_visits() - before
    };
    assert_eq!(visits(""), visits("::options digits 9"));
}
