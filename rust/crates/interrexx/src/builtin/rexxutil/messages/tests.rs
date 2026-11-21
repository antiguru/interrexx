use super::{
    CATALOG_MAGIC, Catalog, CatalogImage, rexx_message, search_candidates, strerror, substitute,
};

/// A catalog as gencat lays one out: `(set, number, text)` entries hashed
/// into `plane_size` buckets, the host-order table, the swapped table, then
/// the messages.
fn catalog_bytes(plane_size: u32, entries: &[(u32, u32, &str)]) -> Vec<u8> {
    let size = plane_size as usize;
    let mut buckets: Vec<Vec<(u32, u32, u32)>> = vec![Vec::new(); size];
    let mut strings = Vec::new();
    for (set, number, text) in entries {
        let offset = u32::try_from(strings.len()).expect("small");
        buckets[((set + 1) * number) as usize % size].push((set + 1, *number, offset));
        strings.extend_from_slice(text.as_bytes());
        strings.push(0);
    }
    let depth = buckets.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let mut table = vec![0u32; size * depth * 3];
    for (bucket, held) in buckets.iter().enumerate() {
        for (level, (set, number, offset)) in held.iter().enumerate() {
            let at = (level * size + bucket) * 3;
            table[at..at + 3].copy_from_slice(&[*set, *number, *offset]);
        }
    }
    let mut bytes = Vec::new();
    for word in [
        CATALOG_MAGIC,
        plane_size,
        u32::try_from(depth).expect("small"),
    ] {
        bytes.extend_from_slice(&word.to_ne_bytes());
    }
    for word in &table {
        bytes.extend_from_slice(&word.to_ne_bytes());
    }
    for word in &table {
        bytes.extend_from_slice(&word.swap_bytes().to_ne_bytes());
    }
    bytes.extend_from_slice(&strings);
    bytes
}

fn image(bytes: Vec<u8>) -> CatalogImage {
    CatalogImage { bytes }
}

fn message(catalog: &Catalog, set: i32, number: i32) -> Option<String> {
    catalog
        .message(set, number)
        .map(|text| String::from_utf8(text).expect("ASCII"))
}

#[test]
fn a_catalog_answers_each_message_by_set_and_number() {
    let entries = [
        (1, 1, "one"),
        (1, 2, "two"),
        (2, 5, "five"),
        (100, 1, "far"),
    ];
    let catalog = Catalog::new(image(catalog_bytes(2, &entries))).expect("valid");
    assert_eq!(message(&catalog, 1, 1).as_deref(), Some("one"));
    assert_eq!(message(&catalog, 1, 2).as_deref(), Some("two"));
    assert_eq!(message(&catalog, 2, 5).as_deref(), Some("five"));
    assert_eq!(message(&catalog, 100, 1).as_deref(), Some("far"));
    assert_eq!(message(&catalog, 1, 3), None);
    assert_eq!(message(&catalog, 3, 1), None);
    assert_eq!(message(&catalog, i32::MAX, 1), None);
    assert_eq!(message(&catalog, 1, -1), None);
}

#[test]
fn a_catalog_header_in_the_other_byte_order_still_opens() {
    let mut bytes = catalog_bytes(1, &[(1, 1, "one")]);
    for at in [0, 4, 8] {
        bytes[at..at + 4].reverse();
    }
    let catalog = Catalog::new(image(bytes)).expect("valid");
    assert_eq!(message(&catalog, 1, 1).as_deref(), Some("one"));
}

/// SF #2168: each damage the oracle answers "Invalid message catalog" for,
/// measured on a gencat catalog.
#[test]
fn a_damaged_catalog_is_rejected() {
    let sound = catalog_bytes(2, &[(1, 1, "one"), (1, 2, "two"), (2, 5, "five")]);
    let word = |bytes: &mut Vec<u8>, at: usize, value: u32| {
        bytes[at..at + 4].copy_from_slice(&value.to_ne_bytes());
    };
    let rejected = |bytes: Vec<u8>| Catalog::new(image(bytes)).is_none();
    assert!(!rejected(sound.clone()));
    let mut bytes = sound.clone();
    bytes.pop();
    assert!(rejected(bytes), "a last message without its NUL");
    let mut bytes = sound.clone();
    bytes.extend_from_slice(b"x");
    assert!(rejected(bytes), "a byte after the last NUL");
    let mut bytes = sound.clone();
    bytes.extend_from_slice(b"x\0");
    assert!(
        !rejected(bytes),
        "bytes after the last message, ending in a NUL"
    );
    let mut bytes = sound.clone();
    word(&mut bytes, 4, 0);
    assert!(rejected(bytes), "no buckets");
    let mut bytes = sound.clone();
    word(&mut bytes, 8, 0);
    assert!(rejected(bytes), "no planes");
    let mut bytes = sound.clone();
    word(&mut bytes, 4, 100_000);
    assert!(rejected(bytes), "tables longer than the file");
    let mut bytes = sound.clone();
    word(&mut bytes, 4, u32::MAX);
    word(&mut bytes, 8, u32::MAX);
    assert!(rejected(bytes), "a table size that overflows");
    assert!(rejected(sound[..12].to_vec()), "a header alone");
}

/// The bound on an offset: the last one that is a byte of the file opens, the
/// next does not; an empty slot's offset is checked too, and only the
/// host-order table is read.
#[test]
fn every_host_order_offset_is_inside_the_file() {
    let sound = catalog_bytes(2, &[(1, 1, "one"), (1, 2, "two"), (2, 5, "five")]);
    let strings = 12 + 2 * 2 * 2 * 12;
    let room = u32::try_from(sound.len() - strings).expect("small");
    let host = if cfg!(target_endian = "little") {
        12
    } else {
        12 + 2 * 2 * 12
    };
    let other = if cfg!(target_endian = "little") {
        12 + 2 * 2 * 12
    } else {
        12
    };
    let with = |table: usize, entry: usize, field: usize, value: u32| {
        let mut bytes = sound.clone();
        let at = table + entry * 12 + field * 4;
        bytes[at..at + 4].copy_from_slice(&value.to_ne_bytes());
        Catalog::new(image(bytes))
    };
    assert!(with(host, 0, 2, room - 1).is_some(), "the last byte");
    assert!(with(host, 0, 2, room).is_none());
    let empty = (0..4)
        .find(|entry| sound[host + entry * 12..host + entry * 12 + 4] == [0; 4])
        .expect("an empty slot");
    assert!(with(host, empty, 2, room).is_none(), "an empty slot");
    assert!(with(other, 0, 2, u32::MAX).is_some(), "the other table");
}

#[test]
fn the_search_path_expands_each_element() {
    let expand = |path: &str, locale: &str| -> Vec<String> {
        search_candidates(path.as_bytes(), b"m.cat", locale.as_bytes())
            .into_iter()
            .map(|candidate| String::from_utf8(candidate).expect("ASCII"))
            .collect()
    };
    assert_eq!(
        expand("/a/%L/%N:/b/%l/%t/%c/%N:", "de_CH.UTF-8"),
        ["/a/de_CH.UTF-8/m.cat", "/b/de/CH/UTF-8/m.cat"]
    );
    assert_eq!(expand("::/c/%N:", "C"), ["m.cat", "m.cat", "/c/m.cat"]);
    assert_eq!(expand("/d/%q/%N:/e/100%%:", "C"), ["/e/100%"]);
    assert_eq!(expand("/f/%t%c:", "C"), ["/f/"]);
    assert_eq!(expand("/g/%t:", "de_"), ["/g/\0"]);
    assert_eq!(expand("/h/%c:", "de."), ["/h/\0"]);
}

#[test]
fn rexx_messages_are_found_by_message_number() {
    // The answer is the catalogue's own template, inserts still marked.
    let template = |major, sub| {
        Some(
            interrexx_inventory::errors::lookup(major, sub)
                .unwrap()
                .text,
        )
    };
    assert_eq!(rexx_message(5), template(5, 0));
    assert_eq!(rexx_message(678), template(88, 907));
    assert_eq!(rexx_message(0), None);
    assert_eq!(rexx_message(-1), None);
}

#[test]
fn substitution_replaces_ampersand_digits_only() {
    let values = [b"abc".to_vec(), Vec::new(), b"&1".to_vec()];
    assert_eq!(substitute(b"&1 + &2 + &3 + &4", &values), b"abc +  + &1 + ");
    assert_eq!(substitute(b"&0 && &", &values), b"&0 && &");
    assert_eq!(substitute(b"&&1&", &values), b"&abc&");
}

/// The text is the standard library's error display less its own suffix,
/// which therefore has to be there for every number.
#[test]
fn strerror_text_drops_exactly_the_standard_library_suffix() {
    for number in (-300..=300).chain([i32::MIN, i32::MAX]) {
        let rendered = std::io::Error::from_raw_os_error(number).to_string();
        assert!(
            rendered.ends_with(&format!(" (os error {number})")),
            "{rendered}"
        );
    }
    assert_eq!(strerror(2), b"No such file or directory");
    // The C library's text for an unknown number is the host's (`docs/design/decisions.md`).
    let unknown: &[u8] = if cfg!(target_vendor = "apple") {
        b"Unknown error: -1"
    } else {
        b"Unknown error -1"
    };
    assert_eq!(strerror(-1), unknown);
}

/// `corpus/oracle-crashes.txt` entry 41: the oracle formats the message with
/// the raising value's null pointer and dies. This crate answers the raise,
/// trapped as any raise from a routine is.
#[test]
fn a_raising_value_the_message_refers_to_is_trapped() {
    let source = b"signal on syntax\n\
        say SysFormatMessage('&1', .array~of(.k~new))\n\
        exit\n\
        syntax: say 'trapped' condition('O')~code condition('O')~message\n\
        exit\n\
        ::class k\n\
        ::method string\n\
        raise syntax 93.900 array('boom')\n";
    let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&outcome.stderr), "");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "trapped 93.900 boom.\n"
    );
}

/// `corpus/oracle-crashes.txt` entry 43: the oracle reads the string through
/// the array layout.
#[test]
fn a_make_array_answering_a_string_is_not_an_array() {
    let source = b"signal on syntax\n\
        say SysFormatMessage('&1', .k~new)\n\
        exit\n\
        syntax: say 'trapped' condition('O')~code\n\
        exit\n\
        ::class k\n\
        ::method makearray\n\
        return 'not an array'\n";
    let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&outcome.stdout), "trapped 98.913\n");
}

/// **The array a `MAKEARRAY` answers is held while its values are
/// converted**: nothing else refers to a fresh one, and each value's `STRING`
/// allocates. Collecting at every allocation, an unheld array loses its
/// values. Oracle rc 0, these lines, measured.
#[test]
fn a_fresh_make_array_answer_outlives_its_values_string_methods() {
    let source = b"say SysFormatMessage('&1 / &2 / &3', .k~new)\n\
        ::class k\n\
        ::method makearray\n\
        return .array~of(.v~new(copies('first', 4)), .v~new(copies('second', 4)),\
        .v~new(copies('third', 4)))\n\
        ::class v\n\
        ::method init\n\
        expose t\n\
        use arg t\n\
        ::method string\n\
        expose t\n\
        x = copies('filler ', 50)\n\
        return t\n";
    let outcome = crate::run_program_collect_every_alloc(
        "/t.rex",
        source.to_vec(),
        crate::Invocation::none(),
    );
    assert!(outcome.collections > 0, "the mode collected nothing");
    assert_eq!(
        outcome.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "firstfirstfirstfirst / secondsecondsecondsecond / thirdthirdthirdthird\n"
    );
}
