//! Every `InstructionKind`, `NodeKind`, `LoopKind`, `PrefixOp`, `EndStyle`,
//! `Trace` and `Operator` variant is either constructed by a program in
//! `rust/corpus/differential.txt`, or is unreachable. The
//! differential half -- those programs agree with the oracle -- is
//! `tests/corpus.rs`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use interrexx_parse::{
    CodeBody, DirectiveKind, EndTarget, Instruction, InstructionKind, Node, NodeKind, Program,
    parse_program,
};

#[path = "variant_tags.rs"]
mod variant_tags;
use variant_tags::{
    Coverage, END_STYLE_TAGS, INSTRUCTION_TAGS, LOOP_TAGS, NODE_TAGS, OPERATOR_TAGS,
    PREFIX_OP_TAGS, TRACE_TAGS, end_style_tag, instruction_tag, loop_tag, node_tag, operator_tag,
    prefix_op_tag, trace_tag,
};

// ---------------------------------------------------------------------------
// The walk. Trimmed from `interrexx-parse/tests/gate_walk/mod.rs` to what the
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

/// The main body **and every `::ROUTINE`, `::METHOD` and `::ATTRIBUTE`
/// body**.
fn each_body<'a>(p: &'a Program, visit: &mut impl FnMut(&'a CodeBody)) {
    visit(&p.main);
    for directive in &p.directives {
        let body = match &directive.kind {
            DirectiveKind::Routine(routine) => routine.body.as_deref(),
            DirectiveKind::Method(method) => method.body.as_deref(),
            DirectiveKind::Attribute(attribute) => attribute.body.as_deref(),
            _ => None,
        };
        if let Some(body) = body {
            visit(body);
        }
    }
}

/// Every instruction of every body `each_body` visits.
fn each_instruction<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Instruction)) {
    each_body(p, &mut |body| {
        body.instructions.iter().for_each(&mut *visit)
    });
}

/// Every expression node of every body `each_body` visits: the subtrees of a
/// body's roots cover its arena once, so this is a loop over each arena.
fn each_node<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Node)) {
    each_body(p, &mut |body| {
        for i in &body.instructions {
            i.kind
                .for_each_root(&mut |root| body.exprs.range(root).iter().for_each(&mut *visit));
        }
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
    let a = dir.join("interrexx-read-subset-test-a.txt");
    let b = dir.join("interrexx-read-subset-test-b.txt");
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
    let mut nodes = Coverage::new("NodeKind", NODE_TAGS);
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
        each_node(&p, &mut |node| {
            nodes.seen.insert(node_tag(node.kind()).0);
            match node.kind() {
                NodeKind::Prefix { op, .. } => {
                    prefix_ops.seen.insert(prefix_op_tag(op).0);
                }
                NodeKind::Binary { op, .. } => {
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
        &nodes,
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
use interrexx_inventory::builtins::EXCLUDED as EXCLUDED_BUILTINS;

#[test]
fn the_builtin_exclusion_set_matches_the_committed_file() {
    let names = interrexx_inventory::builtins::NAMES;
    assert_eq!(
        names.len(),
        84,
        "BuiltinFunctions.cpp's table moved off 84 entries -- this test's own \
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
    for partial in interrexx_inventory::builtins::PARTIALLY_EXCLUDED {
        assert!(
            EXCLUDED_BUILTINS.iter().any(|(name, _, _)| name == partial),
            "{partial} is a partial exclusion but is not in EXCLUDED at all"
        );
    }
    assert_eq!(
        interrexx_inventory::builtins::wholly_excluded().len(),
        0,
        "the whole exclusions are EXCLUDED less the partial rows"
    );
    assert_eq!(
        interrexx_inventory::builtins::PARTIALLY_EXCLUDED.len(),
        0,
        "no builtin is in scope in one form and out of it in another"
    );

    // A later-level name is in the table, and neither excluded nor in scope.
    let later = interrexx_inventory::builtins::LATER_LEVEL;
    for (name, _level) in later {
        assert!(
            names.contains(name),
            "{name} is listed as of a later level but is not in \
             BuiltinFunctions.cpp's builtin table at all"
        );
        assert!(
            !EXCLUDED_BUILTINS
                .iter()
                .any(|(excluded, _, _)| excluded == name),
            "{name} is listed both as of a later level and as excluded"
        );
    }

    // Derived, not asserted: the number is read from the table rather than
    // copied.
    let in_scope =
        names.len() - interrexx_inventory::builtins::wholly_excluded().len() - later.len();
    assert_eq!(
        in_scope, 81,
        "all 81 builtins of this crate's level are in scope"
    );
    assert_eq!(
        interrexx_inventory::builtins::in_scope().len(),
        in_scope,
        "in_scope() must enumerate exactly the names that arithmetic counts"
    );
}
