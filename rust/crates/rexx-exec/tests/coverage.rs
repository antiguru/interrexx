//! Every `InstructionKind`, `ExprKind`, `LoopKind`, `PrefixOp`, `EndStyle`,
//! `Trace` and `Operator` variant is either constructed by a program in
//! `rust/corpus/differential.txt`, or is unreachable. The
//! differential half -- those programs agree with the oracle -- is
//! `tests/corpus.rs`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rexx_parse::{
    DirectiveKind, EndTarget, Expr, ExprKind, Instruction, InstructionKind, Loop, LoopKind,
    Program, Trace, parse_program,
};

#[path = "variant_tags.rs"]
mod variant_tags;
use variant_tags::{
    Coverage, END_STYLE_TAGS, EXPR_TAGS, INSTRUCTION_TAGS, LOOP_TAGS, OPERATOR_TAGS,
    PREFIX_OP_TAGS, TRACE_TAGS, end_style_tag, expr_tag, instruction_tag, loop_tag, operator_tag,
    prefix_op_tag, trace_tag,
};

// ---------------------------------------------------------------------------
// The walk. Trimmed from `rexx-parse/tests/gate_walk/mod.rs` to what the
// subset actually contains: the bodies of `::ROUTINE`, `::METHOD` and
// `::ATTRIBUTE`, plus `::CLASS` and `::CONSTANT` admitted as directives that
// carry no body. `assert_program_has_only_admitted_directives` guards that
// assumption at every parse rather than silently under-walking a program
// that gained a directive kind this walker does not know about.
// ---------------------------------------------------------------------------

/// Whether `kind` is one of the directive kinds this walker admits without
/// panicking: `::ROUTINE`, `::METHOD` and `::ATTRIBUTE`, each walked into by
/// `each_instruction` below, plus the ones that own no body for it to walk.
fn is_admitted_directive_kind(kind: &DirectiveKind) -> bool {
    match kind {
        DirectiveKind::Routine(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Method(_)
        | DirectiveKind::Attribute(_)
        | DirectiveKind::Constant(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Annotate(_)
        | DirectiveKind::Requires(_)
        // `::RESOURCE` owns lines rather than instructions, so `each_instruction`'s
        // own `_ => None` already walks all of it there is to walk.
        | DirectiveKind::Resource(_) => true,
    }
}

fn assert_program_has_only_admitted_directives(path: &Path, p: &Program) {
    assert_directives_admitted_by(path, p, is_admitted_directive_kind);
}

/// [`assert_program_has_only_admitted_directives`] with the predicate as a
/// parameter.
fn assert_directives_admitted_by(path: &Path, p: &Program, admit: fn(&DirectiveKind) -> bool) {
    let others: Vec<&str> = p
        .directives
        .iter()
        .filter(|d| !admit(&d.kind))
        .map(|d| d.kind.keyword())
        .collect();
    assert!(
        others.is_empty(),
        "{} has a `::` directive this walker does not admit ({others:?}) -- see \
         `is_admitted_directive_kind` for the set and `each_instruction` for \
         which of them carry a body it descends into; either the subset gained \
         a directive kind by mistake or this walker needs widening",
        path.display()
    );
}

/// Every one of `DirectiveKind`'s nine keywords, checked against
/// [`is_admitted_directive_kind`]'s real answer for a real parsed instance of
/// that kind -- not only `::ATTRIBUTE`, which `an_unadmitted_directive_still_panics`
/// below covers alone.
#[test]
fn every_directive_keyword_is_correctly_admitted_or_refused() {
    let cases: &[(&str, &str, bool)] = &[
        ("::annotate package\n", "ANNOTATE", true),
        ("::attribute a\n", "ATTRIBUTE", true),
        ("::class c\n", "CLASS", true),
        ("::constant k 1\n", "CONSTANT", true),
        ("::method m\n  return 1\n", "METHOD", true),
        ("::options noprolog\n", "OPTIONS", true),
        ("::requires \"nosuch\"\n", "REQUIRES", true),
        ("::resource d\nbody\n::END\n", "RESOURCE", true),
        ("::routine r\n  return 1\n", "ROUTINE", true),
    ];
    for (text, expected_keyword, expected_admitted) in cases {
        let p = parse_program(text.as_bytes().to_vec())
            .unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"));
        assert_eq!(
            p.directives.len(),
            1,
            "{text:?} did not produce exactly one directive"
        );
        let kind = &p.directives[0].kind;
        assert_eq!(
            kind.keyword(),
            *expected_keyword,
            "{text:?} parsed to the wrong directive kind"
        );
        assert_eq!(
            is_admitted_directive_kind(kind),
            *expected_admitted,
            "{expected_keyword} admission disagrees with the committed expectation"
        );
    }
}

/// `::CLASS K` is an inline literal here, not a corpus entry, so that the
/// parser admits it without a corpus program depending on it.
#[test]
fn a_class_directive_is_admitted_without_panicking() {
    let p = parse_program(b"::CLASS K\n".to_vec()).expect("::CLASS K parses");
    assert_program_has_only_admitted_directives(Path::new("<inline>"), &p);
}

/// **[`each_instruction`] descends into a `::METHOD` and a `::ATTRIBUTE`
/// body**, which is what makes a construct written only inside one count
/// toward criterion 1.
#[test]
fn the_walker_descends_into_a_method_body_and_an_attribute_body() {
    let p = parse_program(
        b"nop\n\
          ::routine r\n  iterate\n\
          ::class K\n\
          ::method m class\n  leave\n\
          ::attribute a get\n  return 1\n"
            .to_vec(),
    )
    .expect("the program parses");
    let mut seen: Vec<&str> = Vec::new();
    each_instruction(&p, &mut |i| seen.push(instruction_tag(&i.kind).0));
    assert_eq!(
        seen,
        vec!["Nop", "Iterate", "Leave", "Return"],
        "the walk did not visit exactly the main body's instruction and one \
         from each directive body, in directive order"
    );
}

/// Pairs with the success above: a directive kind the predicate refuses must
/// still panic, or a widening silently removed the guard rather than widening
/// it.
#[test]
#[should_panic(expected = "has a `::` directive this walker does not admit")]
fn an_unadmitted_directive_still_panics() {
    let p = parse_program(b"::RESOURCE d\nbody\n::END\n".to_vec()).expect("::RESOURCE parses");
    assert_directives_admitted_by(Path::new("<inline>"), &p, |kind| {
        !matches!(kind, DirectiveKind::Resource(_))
    });
}

/// Every direct child expression of `expr`, in source order. Exhaustive so a
/// new `ExprKind` variant is a compile error here, the same guarantee the
/// `tags!` tables above carry.
fn children_of<'a>(expr: &'a Expr, f: &mut impl FnMut(&'a Expr)) {
    match &expr.kind {
        ExprKind::Literal(_)
        | ExprKind::Constant(_)
        | ExprKind::Variable(_)
        | ExprKind::Stem(_)
        | ExprKind::Compound(_)
        | ExprKind::DotVariable(_)
        | ExprKind::ClassResolver { .. } => {}
        ExprKind::Prefix { operand, .. } => f(operand),
        ExprKind::Binary { left, right, .. } => {
            f(left);
            f(right);
        }
        ExprKind::Call { args, .. } | ExprKind::QualifiedCall { args, .. } => {
            for arg in args.iter().flatten() {
                f(arg);
            }
        }
        ExprKind::Message {
            target,
            super_class,
            args,
            ..
        } => {
            f(target);
            if let Some(super_class) = super_class {
                f(super_class);
            }
            for arg in args.iter().flatten() {
                f(arg);
            }
        }
        ExprKind::List(items) => {
            for item in items.iter().flatten() {
                f(item);
            }
        }
        ExprKind::Logical(items) => {
            for item in items {
                f(item);
            }
        }
        ExprKind::VariableReference(inner) => f(inner),
    }
}

/// Every top-level expression an instruction holds directly. Exhaustive over
/// `InstructionKind`.
fn exprs_of_instruction<'a>(kind: &'a InstructionKind, f: &mut impl FnMut(&'a Expr)) {
    let opt = |e: &'a Option<Expr>, f: &mut dyn FnMut(&'a Expr)| {
        if let Some(e) = e {
            f(e);
        }
    };
    match kind {
        InstructionKind::Assignment { target, value } => {
            f(target);
            f(value);
        }
        InstructionKind::Message { term, value } => {
            f(term);
            opt(value, f);
        }
        InstructionKind::Command { expression }
        | InstructionKind::Push { expression }
        | InstructionKind::Queue { expression }
        | InstructionKind::Say { expression }
        | InstructionKind::Return { expression }
        | InstructionKind::Exit { expression }
        | InstructionKind::Reply { expression }
        | InstructionKind::Numeric { expression, .. } => opt(expression, f),
        InstructionKind::Do(l) | InstructionKind::Loop(l) => exprs_of_loop(l, f),
        InstructionKind::If { condition, .. } | InstructionKind::When { condition, .. } => {
            f(condition)
        }
        InstructionKind::WhenCase { values, .. } => {
            for v in values {
                f(v);
            }
        }
        InstructionKind::Select { case, .. } => opt(case, f),
        InstructionKind::Label { .. }
        | InstructionKind::Then
        | InstructionKind::Else { .. }
        | InstructionKind::Otherwise
        | InstructionKind::Leave { .. }
        | InstructionKind::Iterate { .. }
        | InstructionKind::End { .. }
        | InstructionKind::Drop { .. }
        | InstructionKind::Expose { .. }
        | InstructionKind::Procedure { .. }
        | InstructionKind::Nop => {}
        InstructionKind::Parse(p) | InstructionKind::Arg(p) | InstructionKind::Pull(p) => {
            for trigger in p.template.iter().flatten() {
                if let Some(e) = &trigger.value {
                    f(e);
                }
                for target in trigger.targets.iter().flatten() {
                    f(target);
                }
            }
            if let rexx_parse::ParseSource::Value(Some(e)) = &p.source {
                f(e);
            }
        }
        InstructionKind::Call(c) => match &**c {
            rexx_parse::Call::Named { args, .. } | rexx_parse::Call::Qualified { args, .. } => {
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            rexx_parse::Call::Dynamic { target, args } => {
                f(target);
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            rexx_parse::Call::Trap(_) => {}
        },
        InstructionKind::Signal(s) => match &**s {
            rexx_parse::Signal::Value(e) => f(e),
            rexx_parse::Signal::Label(_) | rexx_parse::Signal::Trap(_) => {}
        },
        InstructionKind::Interpret { expression } | InstructionKind::Options { expression } => {
            f(expression)
        }
        InstructionKind::Guard(g) => opt(&g.condition, f),
        InstructionKind::Forward(fw) => {
            opt(&fw.to, f);
            opt(&fw.message, f);
            opt(&fw.class, f);
            opt(&fw.arguments, f);
            if let Some(items) = &fw.array {
                for item in items.iter().flatten() {
                    f(item);
                }
            }
        }
        InstructionKind::Raise(r) => {
            opt(&r.rc, f);
            opt(&r.description, f);
            opt(&r.additional, f);
            if let Some(items) = &r.array {
                for item in items.iter().flatten() {
                    f(item);
                }
            }
            if let Some(result) = &r.result {
                opt(&result.value, f);
            }
        }
        InstructionKind::Use(u) => match &**u {
            rexx_parse::Use::Arg { targets, .. } => {
                for t in targets.iter().flatten() {
                    f(&t.target);
                    opt(&t.default, f);
                }
            }
            rexx_parse::Use::Local { .. } => {}
        },
        InstructionKind::Address(a) => {
            opt(&a.dynamic, f);
            opt(&a.command, f);
            if let Some(io) = &a.io {
                for r in [&io.input, &io.output, &io.error] {
                    match r {
                        rexx_parse::Redirection::Stream(e) | rexx_parse::Redirection::Using(e) => {
                            f(e)
                        }
                        rexx_parse::Redirection::Default
                        | rexx_parse::Redirection::Normal
                        | rexx_parse::Redirection::Stem(_) => {}
                    }
                }
            }
        }
        InstructionKind::Trace(t) => {
            if let Trace::Value(e) = t {
                f(e);
            }
        }
    }
}

fn exprs_of_loop<'a>(l: &'a Loop, f: &mut impl FnMut(&'a Expr)) {
    match &l.kind {
        LoopKind::Simple | LoopKind::Forever => {}
        LoopKind::Count(e) => {
            if let Some(e) = e {
                f(e);
            }
        }
        LoopKind::Controlled(c) => {
            f(&c.initial);
            for e in [&c.to, &c.by, &c.for_count].into_iter().flatten() {
                f(e);
            }
        }
        LoopKind::Over {
            target, for_count, ..
        }
        | LoopKind::With {
            target, for_count, ..
        } => {
            f(target);
            if let Some(e) = for_count {
                f(e);
            }
        }
    }
    if let Some(cond) = &l.conditional {
        f(&cond.condition);
    }
}

/// Every instruction of the main body **and of every `::ROUTINE` body**.
fn each_instruction<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Instruction)) {
    for i in &p.main.instructions {
        visit(i);
    }
    for directive in &p.directives {
        let body = match &directive.kind {
            DirectiveKind::Routine(routine) => routine.body.as_ref(),
            DirectiveKind::Method(method) => method.body.as_ref(),
            DirectiveKind::Attribute(attribute) => attribute.body.as_ref(),
            _ => None,
        };
        if let Some(body) = body {
            for i in &body.instructions {
                visit(i);
            }
        }
    }
}

fn each_expr<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Expr)) {
    // An explicit stack, not recursion through `children_of`: `deep_nested_expr.rex`'s
    // 3,000-term chain is in this subset specifically to exercise this walker, and
    // `variants.rs` measured that a naive recursive walk aborts on
    // it. See that file's own comment for the full argument.
    fn walk<'a>(root: &'a Expr, visit: &mut impl FnMut(&'a Expr)) {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            visit(e);
            let mut children: Vec<&'a Expr> = Vec::new();
            children_of(e, &mut |child| children.push(child));
            stack.extend(children.into_iter().rev());
        }
    }
    each_instruction(p, &mut |i| {
        exprs_of_instruction(&i.kind, &mut |e| walk(e, visit));
    });
}

/// The union of every non-comment, non-blank line across `list_paths`, in
/// first-seen order, each entry appearing once even if two files name the
/// same corpus program.
fn read_subset(list_paths: &[&Path]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut union = Vec::new();
    for list_path in list_paths {
        let text = fs::read_to_string(list_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if seen.insert(line.to_string()) {
                union.push(line.to_string());
            }
        }
    }
    union
}

/// `read_subset`'s multi-file union and de-duplication: first-seen order
/// across files, and a name repeated in the second file appearing only once,
/// at its *first* position.
#[test]
fn read_subset_unions_two_files_first_seen_order_deduplicated() {
    let dir = std::env::temp_dir();
    let a = dir.join("rexx-exec-read-subset-test-a.txt");
    let b = dir.join("rexx-exec-read-subset-test-b.txt");
    fs::write(&a, "# a comment\none.rex\ntwo.rex\n").expect("writing the first list file");
    fs::write(&b, "two.rex\nthree.rex\n").expect("writing the second list file");

    let union = read_subset(&[&a, &b]);
    assert_eq!(
        union,
        vec![
            "one.rex".to_string(),
            "two.rex".to_string(),
            "three.rex".to_string()
        ],
        "the union must be first-seen order across files (one.rex before \
         two.rex before three.rex) with two.rex's second, repeated \
         occurrence (in b) dropped rather than duplicated"
    );

    let _ = fs::remove_file(&a);
    let _ = fs::remove_file(&b);
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// The corpus list the coverage run reads.
const SUBSET_FILES: &[&str] = &["differential.txt"];

/// The subset programs translation refuses, with the error each must fail
/// with. They run none of their instructions, so they witness no variant.
const TRANSLATION_REFUSED: &[(&str, u16, u16)] = &[
    (
        "lang/directive_constant_expression_needs_class.rex",
        99,
        906,
    ),
    ("lang/class_duplicate_method.rex", 99, 902),
    ("lang/class_duplicate_constant_and_method.rex", 99, 902),
    ("lang/class_duplicate_attribute.rex", 99, 931),
    ("lang/class_duplicate_constant.rex", 99, 932),
    ("lang/class_duplicate_class.rex", 99, 901),
    ("lang/directive_annotate_missing_target.rex", 99, 945),
    ("lang/directive_method_external_duplicate_wins.rex", 99, 902),
    ("lang/directive_method_external_source_order.rex", 99, 903),
    (
        "lang/directive_method_external_before_duplicate.rex",
        99,
        902,
    ),
    ("lang/directive_options_digits_below_fuzz.rex", 33, 1),
];

/// The coverage property, read against the whole corpus list.
#[test]
fn every_in_scope_variant_is_witnessed_by_the_corpus_list() {
    let mut instructions = Coverage::new("InstructionKind", INSTRUCTION_TAGS);
    let mut exprs = Coverage::new("ExprKind", EXPR_TAGS);
    let mut loops = Coverage::new("LoopKind", LOOP_TAGS);
    let mut prefix_ops = Coverage::new("PrefixOp", PREFIX_OP_TAGS);
    let mut end_styles = Coverage::new("EndStyle", END_STYLE_TAGS);
    let mut traces = Coverage::new("Trace", TRACE_TAGS);
    let mut operators = Coverage::new("Operator", OPERATOR_TAGS);

    let corpus_dir = corpus_dir();
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    assert!(
        !subset.is_empty(),
        "the corpus list named no programs -- that is a corpus defect, \
         not an empty pass"
    );

    for rel_path in &subset {
        let abs = corpus_dir.join(rel_path);
        let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
        let refused = TRANSLATION_REFUSED
            .iter()
            .find(|(name, _, _)| name == rel_path);
        let p = match (parse_program(text), refused) {
            (Ok(p), None) => p,
            (Err(e), Some(&(_, code, sub))) => {
                assert_eq!(
                    (e.code(), e.sub()),
                    (code, sub),
                    "{} is refused with a different error",
                    abs.display()
                );
                continue;
            }
            (Ok(_), Some(_)) => panic!(
                "{} is listed in TRANSLATION_REFUSED and parsed",
                abs.display()
            ),
            (Err(e), None) => panic!("{} failed to parse: {e:?}", abs.display()),
        };
        assert_program_has_only_admitted_directives(&abs, &p);

        each_instruction(&p, &mut |i| {
            instructions.seen.insert(instruction_tag(&i.kind).0);
            match &i.kind {
                InstructionKind::Do(l) | InstructionKind::Loop(l) => {
                    loops.seen.insert(loop_tag(&l.kind).0);
                }
                InstructionKind::End {
                    closes: Some(EndTarget { style, .. }),
                    ..
                } => {
                    end_styles.seen.insert(end_style_tag(style).0);
                }
                InstructionKind::Trace(t) => {
                    traces.seen.insert(trace_tag(t).0);
                }
                _ => {}
            }
        });
        each_expr(&p, &mut |e| {
            exprs.seen.insert(expr_tag(&e.kind).0);
            match &e.kind {
                ExprKind::Prefix { op, .. } => {
                    prefix_ops.seen.insert(prefix_op_tag(op).0);
                }
                ExprKind::Binary { op, .. } => {
                    operators.seen.insert(operator_tag(op).0);
                }
                _ => {}
            }
        });
    }

    for (name, _, _) in TRANSLATION_REFUSED {
        assert!(
            subset.iter().any(|path| path == name),
            "{name} is listed in TRANSLATION_REFUSED and not in the corpus list"
        );
    }

    let mut report = String::new();
    for cov in [
        &instructions,
        &exprs,
        &loops,
        &prefix_ops,
        &end_styles,
        &traces,
        &operators,
    ] {
        let missing = cov.unwitnessed();
        if !missing.is_empty() {
            use std::fmt::Write as _;
            writeln!(
                report,
                "{}: {} variant(s) unwitnessed by the corpus list: {}",
                cov.category,
                missing.len(),
                missing.join(", ")
            )
            .unwrap();
        }
    }
    assert!(report.is_empty(), "the coverage property fails:\n{report}");
}

/// The builtin exclusions (none), written as a literal rather than parsed
/// from a file -- the same choice `tests/assertions.rs`'s `EXEMPT` list
/// makes. Changing a name there without changing the table is exactly the
/// drift this test exists to catch.
use rexx_inventory::builtins::EXCLUDED as EXCLUDED_BUILTINS;

#[test]
fn the_builtin_exclusion_set_matches_the_committed_file() {
    let names = rexx_inventory::builtins::NAMES;
    assert_eq!(
        names.len(),
        81,
        "BuiltinFunctions.cpp's table moved off 81 entries -- this test's own \
         derivation below needs revisiting"
    );

    let mut seen = HashSet::new();
    for (excluded, _owner, _reason) in EXCLUDED_BUILTINS {
        assert!(
            names.contains(excluded),
            "{excluded} is listed as excluded but is not in \
             BuiltinFunctions.cpp's builtin table at all"
        );
        assert!(
            seen.insert(*excluded),
            "{excluded} is listed twice in EXCLUDED_BUILTINS"
        );
    }
    assert_eq!(
        EXCLUDED_BUILTINS.len(),
        0,
        "the whole exclusions plus the partial rows are empty"
    );

    // Every partial row must also be an excluded row, or `wholly_excluded`
    // subtracts a name that was never in the set and silently returns 16.
    for partial in rexx_inventory::builtins::PARTIALLY_EXCLUDED {
        assert!(
            EXCLUDED_BUILTINS.iter().any(|(name, _, _)| name == partial),
            "{partial} is a partial exclusion but is not in EXCLUDED at all"
        );
    }
    assert_eq!(
        rexx_inventory::builtins::wholly_excluded().len(),
        0,
        "the whole exclusions are EXCLUDED less the partial rows"
    );
    assert_eq!(
        rexx_inventory::builtins::PARTIALLY_EXCLUDED.len(),
        0,
        "no builtin is in scope in one form and out of it in another"
    );

    // Derived, not asserted: the number is read from the table rather than
    // copied.
    let in_scope = names.len() - rexx_inventory::builtins::wholly_excluded().len();
    assert_eq!(in_scope, 81, "all 81 builtins are in scope");
    assert_eq!(
        rexx_inventory::builtins::in_scope().len(),
        in_scope,
        "in_scope() must enumerate exactly the names that arithmetic counts"
    );
}
