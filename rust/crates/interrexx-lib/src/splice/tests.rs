use super::*;

macro_rules! fixture {
    ($name:literal) => {
        include_bytes!(concat!("../../tests/fixtures/", $name))
    };
}

const HOST_A: &[u8] = fixture!("host_a.rex");
const HOST_B: &[u8] = fixture!("host_b.rex");

fn host<'a>(name: &'a str, bytes: &'a [u8]) -> Host<'a> {
    Host { name, bytes }
}

fn rep<'a>(file: &'a str, text: &'a [u8]) -> Replacement<'a> {
    Replacement { file, text }
}

fn allow<'a>(class: &'a str, member: &'a str) -> Dropped<'a> {
    Dropped {
        class,
        member,
        reason: "test",
        sent_elsewhere: false,
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("fixtures are UTF-8")
}

/// The line number (1-based) of the first line that starts with `prefix`.
fn line_of(bytes: &[u8], prefix: &str) -> usize {
    1 + text(bytes)
        .lines()
        .position(|l| l.starts_with(prefix))
        .unwrap_or_else(|| panic!("no line starts with {prefix}"))
}

fn classes_of(bytes: &[u8]) -> Vec<String> {
    let program = parse_program(bytes.to_vec()).expect("spliced host parses");
    program
        .directives
        .iter()
        .filter_map(|d| match &d.kind {
            DirectiveKind::Class(c) => Some(String::from_utf8_lossy(&c.name).into_owned()),
            _ => None,
        })
        .collect()
}

/// With nothing to replace, every host comes back byte for byte.
#[test]
fn no_replacements_leave_every_host_unchanged() {
    let out = splice_all(&[host("A", HOST_A), host("B", HOST_B)], &[], &[]).unwrap();
    assert_eq!(out[0].bytes, HOST_A);
    assert_eq!(out[1].bytes, HOST_B);
    assert!(out.iter().all(|s| s.splices.is_empty()));
    assert_eq!(splice(HOST_A, &[]).unwrap().bytes, HOST_A);
}

/// The replaced class is declared where the old one was, the cut ranges keep
/// their lines, and every later class sits the replacement's line count lower
/// down than before (the replacement's lines in, the cut lines kept blank).
#[test]
fn a_splice_keeps_order_and_the_cut_lines() {
    let replacement = fixture!("rep_alpha.rex");
    let out = splice_all(&[host("A", HOST_A)], &[rep("alpha.rex", replacement)], &[]).unwrap();
    let spliced = &out[0];
    assert_eq!(classes_of(&spliced.bytes), ["ALPHA", "BETA", "LAST"]);
    let block_lines = line_breaks(&spliced.splices[0].range.clone().fold(
        Vec::new(),
        |mut v, i| {
            v.push(spliced.bytes[i]);
            v
        },
    ));
    assert_eq!(
        line_of(&spliced.bytes, "::class Beta"),
        line_of(HOST_A, "::class Beta") + block_lines
    );
    // The routine between two cut members stays, in place after the block.
    let after = text(&spliced.bytes);
    assert!(after.contains("::routine helper"));
    assert!(!after.contains("host hello"));
    assert!(!after.contains("host after"));
    assert!(after.contains("replaced hello"));
    // The record's range is the replacement's text, starting at its directive.
    let range = &spliced.splices[0];
    assert_eq!(range.class, "ALPHA");
    assert_eq!(range.file, "alpha.rex");
    assert_eq!(range.first_line, 2);
    assert!(text(&spliced.bytes[range.range.clone()]).starts_with("::class Alpha public\n"),);
}

/// Everything before the replaced class, and after the cut ranges, is the
/// host's own bytes.
#[test]
fn a_splice_changes_nothing_outside_the_replaced_class() {
    let out = splice(HOST_A, &[rep("alpha.rex", fixture!("rep_alpha.rex"))]).unwrap();
    let spliced = text(&out.bytes);
    assert!(
        spliced.starts_with("/* synthetic host A: the prologue, then classes */\nsay 'prologue'\n")
    );
    assert!(
        spliced.contains("::class Beta public subclass Alpha\n::method bye\n  return 'host bye'\n")
    );
}

/// A replaced class that is last in its host: its member range runs to the end
/// of the host.
#[test]
fn the_last_class_of_a_host_is_replaced_to_the_end() {
    let out = splice(HOST_A, &[rep("last.rex", fixture!("rep_last.rex"))]).unwrap();
    let spliced = text(&out.bytes);
    assert!(spliced.contains("replaced final"));
    assert!(!spliced.contains("host final"));
    assert_eq!(classes_of(&out.bytes), ["ALPHA", "BETA", "LAST"]);
    // The host has the same number of lines, plus the block's.
    let block_breaks = line_breaks(&out.splices[0].range.clone().fold(Vec::new(), |mut v, i| {
        v.push(out.bytes[i]);
        v
    }));
    assert_eq!(line_breaks(&out.bytes), line_breaks(HOST_A) + block_breaks);
}

/// A class of one file in each of two hosts: public classes may split, one
/// non-public class may not.
#[test]
fn classes_of_one_file_split_across_hosts_only_when_all_are_public() {
    let hosts = [host("A", HOST_A), host("B", HOST_B)];
    let out = splice_all(
        &hosts,
        &[rep("two.rex", fixture!("rep_two_hosts.rex"))],
        &[],
    )
    .unwrap();
    assert!(!text(&out[0].bytes).contains("host hello"));
    assert!(text(&out[1].bytes).contains("replaced run"));
    assert_eq!(
        splice_all(
            &hosts,
            &[rep("two.rex", fixture!("rep_two_hosts_private.rex"))],
            &[]
        )
        .unwrap_err(),
        SpliceError::SplitNonPublic {
            file: "two.rex".into()
        }
    );
}

/// A non-public class replaced together with the classes of its own host only.
#[test]
fn a_non_public_class_replaces_in_its_own_host() {
    let out = splice_all(
        &[host("A", HOST_A), host("B", HOST_B)],
        &[rep(
            "hidden.rex",
            b"::class Hidden private\n::method secret\n  return 'replaced secret'\n",
        )],
        &[],
    )
    .unwrap();
    assert!(text(&out[1].bytes).contains("replaced secret"));
    assert_eq!(out[0].bytes, HOST_A);
}

#[test]
fn a_class_no_host_declares_is_refused() {
    let err = splice_all(
        &[host("A", HOST_A)],
        &[rep("ghost.rex", fixture!("rep_ghost.rex"))],
        &[],
    )
    .unwrap_err();
    assert_eq!(
        err,
        SpliceError::ClassNotFound {
            class: "NOSUCH".into()
        }
    );
}

#[test]
fn a_class_two_hosts_declare_is_refused() {
    let twin = b"::class Alpha public\n::method init\n  return 1\n";
    let err = splice_all(
        &[host("A", HOST_A), host("T", twin)],
        &[rep("alpha.rex", fixture!("rep_alpha.rex"))],
        &[],
    )
    .unwrap_err();
    assert_eq!(
        err,
        SpliceError::ClassInSeveralHosts {
            class: "ALPHA".into()
        }
    );
}

#[test]
fn a_class_replaced_by_two_blocks_is_refused() {
    let err = splice_all(
        &[host("A", HOST_A)],
        &[
            rep("one.rex", fixture!("rep_alpha.rex")),
            rep("two.rex", fixture!("rep_alpha.rex")),
        ],
        &[],
    )
    .unwrap_err();
    assert_eq!(
        err,
        SpliceError::DuplicateReplacement {
            class: "ALPHA".into()
        }
    );
}

#[test]
fn a_replacement_holds_only_class_blocks() {
    for (name, bytes, line) in [
        ("main.rex", &fixture!("rep_main_code.rex")[..], 1),
        ("routine.rex", &fixture!("rep_routine.rex")[..], 4),
        (
            "method.rex",
            &b"/* c */\n::method early\n  return 1\n"[..],
            2,
        ),
    ] {
        let err = splice_all(&[host("A", HOST_A)], &[rep(name, bytes)], &[]).unwrap_err();
        assert_eq!(
            err,
            SpliceError::NotClassBlocks {
                file: name.into(),
                line
            },
            "{name}"
        );
    }
}

/// A replacement that does not parse is reported as its file, line and code.
#[test]
fn a_replacement_that_does_not_parse_is_located_in_its_file() {
    let err = splice_all(
        &[host("A", HOST_A)],
        &[rep("syntax.rex", fixture!("rep_syntax.rex"))],
        &[],
    )
    .unwrap_err();
    let SpliceError::ReplacementParse { file, line, .. } = &err else {
        panic!("wrong error {err:?}");
    };
    assert_eq!((file.as_str(), *line), ("syntax.rex", 5));
    // The rendering names the file, the line and the code, nothing else.
    let shown = err.to_string();
    assert!(shown.starts_with("syntax.rex:5: "), "{shown}");
    assert!(!shown.contains("broken"));
}

/// A host that does not parse is located in the host.
#[test]
fn a_host_that_does_not_parse_is_located_in_the_host() {
    let err = splice_all(&[host("A", b"say 'x'\nif then\n")], &[], &[]).unwrap_err();
    let SpliceError::HostParse { file, line, .. } = err else {
        panic!("wrong error");
    };
    assert_eq!((file.as_str(), line), ("A", 2));
}

/// A parse failure at a byte inside a splice is placed in the replacement
/// file, at the line the byte has there.
#[test]
fn a_failure_inside_a_splice_is_placed_in_the_replacement_file() {
    let bytes = b"say 1\n::class A public\n::method m\n  if then\n::class B\n".to_vec();
    let error = parse_program(bytes.clone()).unwrap_err();
    let start = 6;
    let spliced = Spliced {
        bytes,
        splices: vec![Splice {
            class: "A".into(),
            file: "a.rex".into(),
            range: start..start + 36,
            first_line: 10,
        }],
    };
    let err = located("H", &spliced, &error);
    let SpliceError::HostParse { file, line, .. } = err else {
        panic!("wrong error");
    };
    // Line 4 of the host; the splice starts on line 2 of the host, line 10 of
    // the file.
    assert_eq!((file.as_str(), line), ("a.rex", 12));
    let outside = located(
        "H",
        &Spliced {
            splices: vec![],
            ..spliced
        },
        &error,
    );
    let SpliceError::HostParse { file, line, .. } = outside else {
        panic!("wrong error");
    };
    assert_eq!((file.as_str(), line), ("H", 4));
}

#[test]
fn an_annotation_among_a_replaced_classs_members_is_refused() {
    let err = splice_all(
        &[host("Ann", fixture!("host_annotated.rex"))],
        &[rep("alpha.rex", fixture!("rep_alpha.rex"))],
        &[],
    )
    .unwrap_err();
    assert_eq!(
        err,
        SpliceError::Annotated {
            host: "Ann".into(),
            class: "ALPHA".into(),
            member: Some("HELLO".into())
        }
    );
    // An annotation of the class itself.
    let err = splice_all(
        &[host(
            "C",
            b"::class Alpha public\n::annotate class Alpha kind 'x'\n",
        )],
        &[rep("a.rex", b"::class Alpha public\n")],
        &[],
    )
    .unwrap_err();
    assert!(matches!(err, SpliceError::Annotated { member: None, .. }));
    // An annotation under another class is not the replaced class's.
    splice_all(
        &[host(
            "D",
            b"::class Alpha public\n::class Other public\n::method m\n  return 1\n\
              ::annotate method m kind 'x'\n",
        )],
        &[rep("a.rex", b"::class Alpha public\n")],
        &[],
    )
    .unwrap();
}

const SENDER: &[u8] = b"::class Caller public\n::method go\n  use arg o\n  return o~Hello\n";
const ALPHA_WITH_HELLO: &[u8] =
    b"::class Alpha public\n::method hello\n  return 'host hello'\n::method keep\n  return 2\n";
const ALPHA_WITHOUT: &[u8] = b"::class Alpha public\n::method keep\n  return 2\n";

#[test]
fn a_dropped_member_needs_an_entry() {
    let err = splice_all(
        &[host("A", ALPHA_WITH_HELLO)],
        &[rep("a.rex", ALPHA_WITHOUT)],
        &[],
    )
    .unwrap_err();
    assert_eq!(
        err,
        SpliceError::MemberDropped {
            class: "ALPHA".into(),
            member: "HELLO".into()
        }
    );
    let out = splice_all(
        &[host("A", ALPHA_WITH_HELLO)],
        &[rep("a.rex", ALPHA_WITHOUT)],
        &[allow("Alpha", "hello")],
    )
    .unwrap();
    assert!(!text(&out[0].bytes).contains("host hello"));
}

#[test]
fn an_entry_for_a_member_the_replacement_keeps_is_refused() {
    let err = splice_all(
        &[host("A", ALPHA_WITH_HELLO)],
        &[rep("a.rex", ALPHA_WITH_HELLO)],
        &[allow("Alpha", "keep")],
    )
    .unwrap_err();
    assert_eq!(
        err,
        SpliceError::DroppedKept {
            class: "ALPHA".into(),
            member: "KEEP".into()
        }
    );
}

#[test]
fn a_dropped_member_sent_by_a_message_term_is_refused() {
    let hosts = [host("A", ALPHA_WITH_HELLO), host("S", SENDER)];
    let err = splice_all(
        &hosts,
        &[rep("a.rex", ALPHA_WITHOUT)],
        &[allow("Alpha", "hello")],
    )
    .unwrap_err();
    assert_eq!(
        err,
        SpliceError::DroppedStillSent {
            class: "ALPHA".into(),
            member: "HELLO".into(),
            host: "S".into()
        }
    );
}

#[test]
fn a_dropped_member_sent_by_a_string_is_refused() {
    for sender in [
        &b"::class Caller public\n::method go\n  use arg o\n  return o~send('hello')\n"[..],
        b"::class Caller public\n::method go\n  use arg o\n  return o~sendWith(\"HELLO\", .array~new)\n",
        b"::class Caller public\n::method go\n  m = .message~new(.nil, 'Foo hello')\n  return m\n",
        b"::class Caller public\n::method go\n  use arg o\n  forward message ('hello')\n",
    ] {
        let hosts = [host("A", ALPHA_WITH_HELLO), host("S", sender)];
        let err = splice_all(&hosts, &[rep("a.rex", ALPHA_WITHOUT)], &[allow("Alpha", "hello")])
            .unwrap_err();
        assert!(
            matches!(err, SpliceError::DroppedStillSent { .. }),
            "{}",
            text(sender)
        );
    }
}

/// The send is found in the same host too, and in a replacement's own code.
#[test]
fn a_send_in_the_hosts_own_or_the_replacements_code_counts() {
    let same: Vec<u8> = [
        ALPHA_WITH_HELLO,
        b"::class Z public\n::method self2\n  return self~hello\n",
    ]
    .concat();
    let err = splice_all(
        &[host("A", &same)],
        &[rep("a.rex", ALPHA_WITHOUT)],
        &[allow("Alpha", "hello")],
    )
    .unwrap_err();
    assert!(matches!(err, SpliceError::DroppedStillSent { .. }));
    let own: &[u8] = b"::class Alpha public\n::method keep\n  return self~hello\n";
    let err = splice_all(
        &[host("A", ALPHA_WITH_HELLO)],
        &[rep("a.rex", own)],
        &[allow("Alpha", "hello")],
    )
    .unwrap_err();
    assert!(matches!(err, SpliceError::DroppedStillSent { .. }));
}

/// A name in a declaration is not a send, and the host code a dropped member
/// held went with it.
#[test]
fn declarations_and_the_cut_code_do_not_count_as_sends() {
    let host_a: Vec<u8> = [
        &b"::class Alpha public\n::method hello\n  return self~hello\n::method keep\n  return 2\n\
          ::class Beta public\n::method 'hello'\n  return 1\n"[..],
    ]
    .concat();
    // `Beta` declares a method of the same name, which is not a send.
    splice_all(
        &[host("A", &host_a)],
        &[rep("a.rex", ALPHA_WITHOUT)],
        &[allow("Alpha", "hello")],
    )
    .unwrap();
}

/// A name that something else is sent is settled in the entry.
#[test]
fn an_entry_can_settle_a_name_that_is_sent_to_something_else() {
    let hosts = [host("A", ALPHA_WITH_HELLO), host("S", SENDER)];
    splice_all(
        &hosts,
        &[rep("a.rex", ALPHA_WITHOUT)],
        &[Dropped {
            sent_elsewhere: true,
            ..allow("Alpha", "hello")
        }],
    )
    .unwrap();
}

/// An attribute's getter and setter are two members, and a send of the
/// attribute's name finds the setter too.
#[test]
fn an_attributes_setter_is_a_member_of_its_own() {
    let host_a = b"::class Alpha public\n::attribute size\n::method keep\n  return 2\n";
    let replacement = b"::class Alpha public\n::attribute size get\n::method keep\n  return 2\n";
    let err = splice_all(&[host("A", host_a)], &[rep("a.rex", replacement)], &[]).unwrap_err();
    assert_eq!(
        err,
        SpliceError::MemberDropped {
            class: "ALPHA".into(),
            member: "SIZE=".into()
        }
    );
}

/// Class methods and instance methods of one name are different members.
#[test]
fn the_class_side_and_the_instance_side_are_different_members() {
    let host_a =
        b"::class Alpha public\n::method new class\n  return 1\n::method new\n  return 2\n";
    let replacement = b"::class Alpha public\n::method new class\n  return 1\n";
    let err = splice_all(&[host("A", host_a)], &[rep("a.rex", replacement)], &[]).unwrap_err();
    assert!(matches!(err, SpliceError::MemberDropped { .. }));
}

#[test]
fn line_helpers_count_cr_lf_and_crlf_as_one_end_each() {
    assert_eq!(line_breaks(b"a\nb\rc\r\nd"), 3);
    assert_eq!(line_at(b"a\nb\rc\r\nd", 8), 4);
    assert_eq!(line_at(b"abc", 2), 1);
}
