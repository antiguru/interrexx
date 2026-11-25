//! The saved macrospace file, interrexx's own layout (Resolution 3): a header of a 16-byte text,
//! an 8-byte signature and an 8-byte count, then per macro its name, position and image, each
//! length-prefixed, little-endian.

use interrexx_cast::CastFrom;

use crate::message::code::{MACRO_FILE_ERROR, OK};
use crate::message::{Macro, macro_position};

/// The header's text. The reference implementation's differs, so its files fail
/// the check.
const TEXT: &[u8; 16] = b"interrexx macros";
const SIGNATURE: u64 = 0x6d61_6372_6f31;
/// The header's size: a file shorter than this cannot be read.
pub const HEADER: usize = 32;
/// `RXMACRO_SIGNATURE_ERROR`.
pub const SIGNATURE_ERROR: u64 = 6;

/// `macros` as a file's bytes.
pub fn write(macros: &[Macro]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER);
    bytes.extend_from_slice(TEXT);
    bytes.extend_from_slice(&SIGNATURE.to_le_bytes());
    bytes.extend_from_slice(&u64::cast_from(macros.len()).to_le_bytes());
    for entry in macros {
        for field in [&entry.name, &entry.image] {
            bytes.extend_from_slice(&u64::cast_from(field.len()).to_le_bytes());
            bytes.extend_from_slice(field);
        }
        bytes.extend_from_slice(&entry.position.to_le_bytes());
    }
    bytes
}

/// The macros a file's bytes hold, in order, or the API code: `RXMACRO_FILE_ERROR` for a file
/// too short for what it promises or with a macro whose position is neither before nor after,
/// `RXMACRO_SIGNATURE_ERROR` for one that is not this layout. The macros read before a failure
/// come with it.
pub fn read(bytes: &[u8]) -> (u64, Vec<Macro>) {
    if bytes.len() < HEADER {
        return (MACRO_FILE_ERROR, Vec::new());
    }
    if &bytes[..16] != TEXT || u64_at(bytes, 16) != Some(SIGNATURE) {
        return (SIGNATURE_ERROR, Vec::new());
    }
    let Some(count) = u64_at(bytes, 24) else {
        return (MACRO_FILE_ERROR, Vec::new());
    };
    let mut at = HEADER;
    let mut macros = Vec::new();
    for _ in 0..count {
        let Some((name, after)) = field(bytes, at) else {
            return (MACRO_FILE_ERROR, macros);
        };
        let Some((image, after)) = field(bytes, after) else {
            return (MACRO_FILE_ERROR, macros);
        };
        let Some(position) = u64_at(bytes, after).filter(|at| macro_position::is_valid(*at)) else {
            return (MACRO_FILE_ERROR, macros);
        };
        at = after + 8;
        macros.push(Macro {
            name: name.to_vec(),
            image: image.to_vec(),
            position,
        });
    }
    (OK, macros)
}

fn u64_at(bytes: &[u8], at: usize) -> Option<u64> {
    let slice = bytes.get(at..at.checked_add(8)?)?;
    Some(u64::from_le_bytes(slice.try_into().ok()?))
}

/// A length-prefixed field at `at`, and where the next begins.
fn field(bytes: &[u8], at: usize) -> Option<(&[u8], usize)> {
    let length = usize::try_from(u64_at(bytes, at)?).ok()?;
    let start = at + 8;
    let end = start.checked_add(length)?;
    Some((bytes.get(start..end)?, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn macro_of(name: &str, image: &str, position: u64) -> Macro {
        Macro {
            name: name.as_bytes().to_vec(),
            image: image.as_bytes().to_vec(),
            position,
        }
    }

    #[test]
    fn a_written_file_reads_back() {
        let macros = vec![macro_of("one", "say 1", 1), macro_of("", "", 2)];
        assert_eq!(read(&write(&macros)), (OK, macros));
        assert_eq!(read(&write(&[])), (OK, Vec::new()));
    }

    #[test]
    fn short_foreign_and_truncated_files_are_refused() {
        assert_eq!(read(b"say 'hello'").0, MACRO_FILE_ERROR);
        let source = b"/* a Rexx program long enough to fill a header */ say 1\n";
        assert_eq!(read(source).0, SIGNATURE_ERROR);
        let mut oracle = b"REXX-ooRexx 6.00".to_vec();
        oracle.extend_from_slice(&0xddd5u64.to_le_bytes());
        oracle.extend_from_slice(&0u64.to_le_bytes());
        assert_eq!(read(&oracle).0, SIGNATURE_ERROR);
        let whole = write(&[macro_of("a", "x", 1), macro_of("b", "yy", 2)]);
        let (status, loaded) = read(&whole[..whole.len() - 3]);
        assert_eq!(
            (status, loaded),
            (MACRO_FILE_ERROR, vec![macro_of("a", "x", 1)])
        );
    }

    /// A macro whose position is neither before nor after stops the read as
    /// a cut file does, with the macros before it.
    #[test]
    fn a_macro_of_no_search_position_stops_the_read() {
        for position in [0, 3, 65537] {
            let bytes = write(&[
                macro_of("a", "x", 2),
                macro_of("b", "y", position),
                macro_of("c", "z", 1),
            ]);
            assert_eq!(
                read(&bytes),
                (MACRO_FILE_ERROR, vec![macro_of("a", "x", 2)]),
                "{position}"
            );
        }
    }
}
