use super::{
    FORMAT, MAGIC, fingerprint, program_from_image, program_from_static_image, program_image,
};
use crate::exprs::{ExprId, NodeKind};
use crate::token::SymbolId;
use crate::{
    CodeBody, ConstantValue, DirectiveKind, InstructionKind, Program, parse_known_program,
    parse_program,
};

/// An image of `FORMAT_3_TEXT`, written by the layout before
/// the expression arenas.
const FORMAT_3_IMAGE: &[u8] = include_bytes!("format3.rxparsed");
const FORMAT_3_TEXT: &[u8] = include_bytes!("format3.rex");

/// Whether two programs hold the same thing in every field, the text, line
/// index and symbol ids included.
fn assert_same(read: &Program, parsed: &Program, what: &str) {
    assert_eq!(read.source.parts(), parsed.source.parts(), "{what}: source");
    assert_eq!(read.symbols.len(), parsed.symbols.len(), "{what}: symbols");
    for index in 0..parsed.symbols.len() {
        let id = SymbolId::from_index(index).expect("an issued id");
        assert_eq!(
            read.symbols.name(id),
            parsed.symbols.name(id),
            "{what}: symbol"
        );
    }
    assert_eq!(read.main, parsed.main, "{what}: main body");
    assert_eq!(read.directives, parsed.directives, "{what}: directives");
}

fn round_trip(text: &[u8], parsed: &Program, what: &str) {
    let image = program_image(parsed);
    let read = program_from_image(&image, text)
        .unwrap_or_else(|| panic!("{what}: its own image is refused"));
    assert_same(&read, parsed, what);
}

/// The image the library's build embeds is accepted, and holds what parsing
/// the library gives, so starting from it is starting from the parse.
#[test]
fn the_librarys_embedded_image_reads_back_as_it_parses() {
    let library = &interrexx_lib::LIBRARY;
    let parsed = parse_known_program(library.source.to_vec()).expect("the library parses");
    let read = program_from_image(library.image, library.source)
        .expect("the embedded image is this build's image of the library");
    assert_same(&read, &parsed, "the embedded library image");
    round_trip(library.source, &parsed, "the library");
}

/// Read from the image built in, the library's code bodies stay encoded until
/// each is first read, and each then reads as the parse has it.
#[test]
fn the_librarys_bodies_decode_when_first_read() {
    let library = &interrexx_lib::LIBRARY;
    let read = program_from_static_image(library.image, library.source)
        .expect("the embedded image is accepted");
    let bodies = || {
        read.directives
            .iter()
            .filter_map(|directive| match &directive.kind {
                DirectiveKind::Method(method) => method.body.as_ref(),
                DirectiveKind::Attribute(attribute) => attribute.body.as_ref(),
                DirectiveKind::Routine(routine) => routine.body.as_ref(),
                _ => None,
            })
    };
    assert!(bodies().count() > 100);
    assert!(bodies().all(|body| !body.is_decoded()));
    let parsed = parse_known_program(library.source.to_vec()).expect("the library parses");
    assert_same(&read, &parsed, "the lazily read library");
    assert!(bodies().all(|body| body.is_decoded()));
}

/// Every corpus program that parses reads back from its image as it parses,
/// which reaches the constructs the library does not use.
#[test]
fn every_corpus_program_reads_back_as_it_parses() {
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let mut directories = vec![corpus];
    let mut programs = 0usize;
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rex") {
                continue;
            }
            let text = std::fs::read(&path).expect("a readable corpus program");
            let Ok(parsed) = parse_program(text.clone()) else {
                continue;
            };
            // The source drops what follows a Ctrl-Z; the image is of what it kept.
            let kept = parsed.source.parts().0.to_vec();
            round_trip(&kept, &parsed, &path.display().to_string());
            programs += 1;
        }
    }
    assert!(programs > 1000, "only {programs} corpus programs parsed");
}

/// An image of other text, of another format, cut short or carrying more is
/// refused rather than read.
#[test]
fn an_image_that_does_not_fit_is_refused() {
    let text = b"say 'a'\n::routine r\nreturn 1\n";
    let parsed = parse_program(text.to_vec()).expect("parses");
    let image = program_image(&parsed);
    assert!(program_from_image(&image, text).is_some());
    assert!(program_from_image(&image, b"say 'b'\n::routine r\nreturn 1\n").is_none());
    assert!(program_from_image(&image, b"say 'a'\n::routine r\nreturn 1\n\n").is_none());
    let mut other_format = image.clone();
    other_format[MAGIC.len()..MAGIC.len() + 4].copy_from_slice(&(FORMAT + 1).to_le_bytes());
    assert!(program_from_image(&other_format, text).is_none());
    for cut in 0..image.len() {
        assert!(
            program_from_image(&image[..cut], text).is_none(),
            "cut at {cut}"
        );
    }
    let mut longer = image.clone();
    longer.push(0);
    assert!(program_from_image(&longer, text).is_none());
}

/// An image of the layout that wrote expression trees, not arenas, is
/// refused, and the text it was made from parses, which is what the caller
/// then does.
#[test]
fn a_format_3_image_is_refused_and_its_text_parses() {
    assert_eq!(
        FORMAT_3_IMAGE[MAGIC.len()..MAGIC.len() + 4],
        3u32.to_le_bytes(),
        "the fixture is a FORMAT 3 image"
    );
    assert_eq!(
        FORMAT_3_IMAGE[MAGIC.len() + 4..MAGIC.len() + 12],
        fingerprint(FORMAT_3_TEXT).to_le_bytes(),
        "the fixture's fingerprint is that of its text, so only the format refuses it"
    );
    assert!(program_from_image(FORMAT_3_IMAGE, FORMAT_3_TEXT).is_none());
    assert!(parse_program(FORMAT_3_TEXT.to_vec()).is_ok());
}

const ARENA_TEXT: &[u8] = b"a = 1 + 2 * 3\nsay a~abs f(a, , 'x') (>a)~value\nexit\n\
f: return arg(1)\n::class c\n::constant k (1 + 2)\n::routine r\nreturn a b c\n";

/// The image of `ARENA_TEXT` parsed and then edited, which is consistent at
/// the byte level and not as an arena where `edit` broke it.
fn edited_image(edit: impl FnOnce(&mut Program)) -> Vec<u8> {
    let mut program = parse_program(ARENA_TEXT.to_vec()).expect("parses");
    edit(&mut program);
    program_image(&program)
}

fn routine_body(program: &mut Program) -> &mut CodeBody {
    program
        .directives
        .iter_mut()
        .find_map(|directive| match &mut directive.kind {
            DirectiveKind::Routine(routine) => routine.body.as_deref_mut(),
            _ => None,
        })
        .expect("the routine has a body")
}

fn say_root(body: &CodeBody) -> ExprId {
    body.instructions
        .iter()
        .find_map(|instruction| match instruction.kind {
            InstructionKind::Say { expression } => expression,
            _ => None,
        })
        .expect("a SAY with an expression")
}

/// An image whose arena breaks an invariant is refused, one case for each
/// kind of invariant, in the main body, a routine's body and a constant's
/// expression alike, while the unedited image of the same text is read.
#[test]
fn an_image_whose_arena_is_inconsistent_is_refused() {
    let parsed = parse_program(ARENA_TEXT.to_vec()).expect("parses");
    let unedited = program_image(&parsed);
    assert!(program_from_image(&unedited, ARENA_TEXT).is_some());
    let past_the_end = |program: &Program| ExprId::from_index(program.main.exprs.len() + 7);
    let cases: Vec<(&str, Vec<u8>)> = vec![
        (
            "a parent link at another node",
            edited_image(|program| {
                let nodes = program.main.exprs.len();
                let id = (0..nodes)
                    .map(ExprId::from_index)
                    .find(|&id| program.main.exprs.node(id).parent().is_some())
                    .expect("a child");
                let other = ExprId::from_index(nodes - 1);
                program.main.exprs.node_mut(id).parent = Some(other);
            }),
        ),
        (
            "a first moved by one",
            edited_image(|program| {
                let root = say_root(&program.main);
                let first = program.main.exprs.node(root).first().index();
                program.main.exprs.node_mut(root).first = ExprId::from_index(first + 1);
            }),
        ),
        (
            "a SAY slot at a node that is not its root",
            edited_image(|program| {
                let root = say_root(&program.main);
                let other = ExprId::from_index(root.index() - 1);
                for instruction in &mut program.main.instructions {
                    if let InstructionKind::Say { expression } = &mut instruction.kind {
                        *expression = Some(other);
                    }
                }
            }),
        ),
        (
            "a child past the node vector",
            edited_image(|program| {
                let beyond = past_the_end(program);
                let exprs = &mut program.main.exprs;
                let id = (0..exprs.len())
                    .map(ExprId::from_index)
                    .find(|&id| matches!(exprs.node(id).kind(), NodeKind::Binary { .. }))
                    .expect("a binary node");
                if let NodeKind::Binary { right, .. } = &mut exprs.node_mut(id).kind {
                    *right = beyond;
                }
            }),
        ),
        (
            "a child after its parent",
            edited_image(|program| {
                let exprs = &mut program.main.exprs;
                let id = (0..exprs.len())
                    .map(ExprId::from_index)
                    .find(|&id| matches!(exprs.node(id).kind(), NodeKind::Binary { .. }))
                    .expect("a binary node");
                let after = ExprId::from_index(id.index() + 1);
                if let NodeKind::Binary { left, .. } = &mut exprs.node_mut(id).kind {
                    *left = after;
                }
            }),
        ),
        (
            "a node reached twice",
            edited_image(|program| {
                let exprs = &mut program.main.exprs;
                let id = (0..exprs.len())
                    .map(ExprId::from_index)
                    .find(|&id| matches!(exprs.node(id).kind(), NodeKind::Binary { .. }))
                    .expect("a binary node");
                if let NodeKind::Binary { left, right, .. } = &mut exprs.node_mut(id).kind {
                    *right = *left;
                }
            }),
        ),
        (
            "a binary operand tagged omitted",
            edited_image(|program| {
                let exprs = &mut program.main.exprs;
                let id = (0..exprs.len())
                    .map(ExprId::from_index)
                    .find_map(|id| match exprs.node(id).kind() {
                        NodeKind::Binary { left, .. } => Some(*left),
                        _ => None,
                    })
                    .expect("a binary node");
                exprs.node_mut(id).kind = NodeKind::Omitted;
            }),
        ),
        (
            "a root that has a parent",
            edited_image(|program| {
                let root = say_root(&program.main);
                program.main.exprs.node_mut(root).parent = Some(ExprId::from_index(0));
            }),
        ),
        (
            "a routine body's root with a parent",
            edited_image(|program| {
                let body = routine_body(program);
                let root = body
                    .instructions
                    .iter()
                    .find_map(|instruction| match instruction.kind {
                        InstructionKind::Return { expression } => expression,
                        _ => None,
                    })
                    .expect("a RETURN with an expression");
                body.exprs.node_mut(root).parent = Some(ExprId::from_index(0));
            }),
        ),
        (
            "a constant's expression with a parent",
            edited_image(|program| {
                for directive in &mut program.directives {
                    if let DirectiveKind::Constant(constant) = &mut directive.kind
                        && let ConstantValue::Expression { exprs, root } = &mut constant.value
                    {
                        exprs.node_mut(*root).parent = Some(ExprId::from_index(0));
                    }
                }
            }),
        ),
    ];
    for (what, image) in cases {
        assert_ne!(image, unedited, "{what}: the edit changed the image");
        assert!(
            program_from_image(&image, ARENA_TEXT).is_none(),
            "{what}: the image is read"
        );
    }
}

/// Whatever one byte of an image is, reading it answers rather than panics.
#[test]
fn an_image_with_a_byte_changed_never_panics_the_reader() {
    let image = program_image(&parse_program(ARENA_TEXT.to_vec()).expect("parses"));
    for at in 0..image.len() {
        for flip in [0xff, 0x01, 0x80] {
            let mut changed = image.clone();
            changed[at] ^= flip;
            let _ = program_from_image(&changed, ARENA_TEXT);
        }
    }
}

/// The image built into the interpreter is not validated: a program read
/// from one holds the arena the image holds, and the body decodes when read.
#[test]
fn a_built_in_image_is_not_validated() {
    let image = edited_image(|program| {
        let root = say_root(&program.main);
        program.main.exprs.node_mut(root).parent = Some(ExprId::from_index(0));
    });
    assert!(program_from_image(&image, ARENA_TEXT).is_none());
    let image: &'static [u8] = Box::leak(image.into_boxed_slice());
    let read = program_from_static_image(image, ARENA_TEXT).expect("a built-in image is read");
    assert!(read.main.check().is_err());
}
