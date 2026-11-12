//! The interpreter's version string and the fields `RexxInfo` cuts from it, the
//! platform name and the line terminator.

/// The platform name `PARSE SOURCE`'s first word carries.
pub(crate) const PLATFORM: &[u8] = b"LINUX";

/// The line terminator `.ENDOFLINE` answers -- measured, `c2x(.endOfLine)`
/// is `0A` here.
pub(crate) const LINE_END: &[u8] = b"\n";

/// `PARSE VERSION`'s string: ooRexx's shape and language level, stamped with this build's date.
pub(crate) const VERSION: &[u8] = concat!(
    "REXX-ooRexx_5.3.0(MT)_64-bit 6.06 ",
    env!("INTERREXX_BUILD_DATE")
)
.as_bytes();

/// `RexxInfo~version`: `ORX_VER.ORX_REL.ORX_MOD`, rendered with the same
/// `%d.%d.%d` that the version string embeds in [`VERSION`].
pub(crate) const VERSION_NUMBER: &[u8] = part(VERSION, UNDERSCORE + 1, PAREN);

/// `RexxInfo~majorVersion`: `ORX_VER`, [`VERSION_NUMBER`]'s first field.
pub(crate) const MAJOR_VERSION: &[u8] = part(VERSION_NUMBER, 0, FIRST_DOT);

/// `RexxInfo~release`: `ORX_REL`, [`VERSION_NUMBER`]'s second field.
pub(crate) const RELEASE: &[u8] = part(VERSION_NUMBER, FIRST_DOT + 1, SECOND_DOT);

/// `RexxInfo~modification`: `ORX_MOD`, [`VERSION_NUMBER`]'s third field.
pub(crate) const MODIFICATION: &[u8] = part(VERSION_NUMBER, SECOND_DOT + 1, VERSION_NUMBER.len());

/// `RexxInfo~languageLevel`: the word the version string carries after the
/// `-bit` one.
pub(crate) const LANGUAGE_LEVEL: &[u8] = part(VERSION, BIT_BLANK + 1, LEVEL_BLANK);

/// `RexxInfo~date`: the interpreter's build date, `day month year`, the tail of
/// [`VERSION`].
pub(crate) const BUILD_DATE: &[u8] = part(VERSION, LEVEL_BLANK + 1, VERSION.len());

/// The pointer width the version string carries in front of `-bit`.
/// `RexxInfo~architecture` reads the pointer size in bits instead and the two
/// must agree, which is what this constant exists to let a test say.
#[cfg(test)]
pub(super) const BIT_WIDTH: &[u8] = part(VERSION, SECOND_UNDERSCORE + 1, DASH);

const UNDERSCORE: usize = seek(VERSION, b'_', 0);
const PAREN: usize = seek(VERSION, b'(', UNDERSCORE);
#[cfg(test)]
const SECOND_UNDERSCORE: usize = seek(VERSION, b'_', PAREN);
#[cfg(test)]
const DASH: usize = seek(VERSION, b'-', SECOND_UNDERSCORE);
const FIRST_DOT: usize = seek(VERSION_NUMBER, b'.', 0);
const SECOND_DOT: usize = seek(VERSION_NUMBER, b'.', FIRST_DOT + 1);
const BIT_BLANK: usize = seek(VERSION, b' ', 0);
const LEVEL_BLANK: usize = seek(VERSION, b' ', BIT_BLANK + 1);

/// The offset of the first `byte` at or after `from`.
const fn seek(bytes: &[u8], byte: u8, from: usize) -> usize {
    let mut at = from;
    while at < bytes.len() {
        if bytes[at] == byte {
            return at;
        }
        at += 1;
    }
    panic!("VERSION does not carry the delimiter its fields are cut at")
}

const fn part(bytes: &'static [u8], from: usize, to: usize) -> &'static [u8] {
    bytes.split_at(to).0.split_at(from).1
}

#[cfg(test)]
#[path = "../build_date.rs"]
mod build_date;

#[cfg(test)]
mod tests {
    use super::build_date::format;

    #[test]
    fn the_date_is_day_month_year_in_utc() {
        assert_eq!(format(0), "1 Jan 1970");
        assert_eq!(format(951_782_400), "29 Feb 2000");
        assert_eq!(format(1_078_012_799), "28 Feb 2004");
        assert_eq!(format(1_078_012_800), "29 Feb 2004");
        assert_eq!(format(1_078_099_200), "1 Mar 2004");
        assert_eq!(format(1_793_836_800), "5 Nov 2026");
        assert_eq!(format(1_793_923_199), "5 Nov 2026");
    }

    #[test]
    fn the_build_date_ends_the_version() {
        assert!(super::VERSION.ends_with(super::BUILD_DATE));
    }
}
