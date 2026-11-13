use super::{FORMAT, MAGIC, program_from_image, program_from_static_image, program_image};
use crate::DirectiveKind;
use crate::token::SymbolId;
use crate::{Program, parse_known_program, parse_program};

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
