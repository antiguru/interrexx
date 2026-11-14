//! Tables mechanically derived from the C++ tree, plus the one hand-written
//! list that indexes into them.
//!
//! # Message texts
//!
//! `messages/<major>.tsv` holds interrexx's own text for each error message
//! of one major, one row per message: `major<TAB>sub<TAB>text`. A text writes
//! insert `k` as `&k` (the utility messages of major 999 too, though the
//! source marks theirs `%k`), and only for a `k` in the row's `rendered` column of
//! `data/errors.tsv`. A line that is empty or starts with `#` is a comment.
//! A text has no tab and no line break. The build checks every file: its
//! name is the major of each of its rows, each code is in `errors.tsv`, no
//! code has two texts, and no text uses an insert outside the rendered set.
//! The texts are checked but not yet used.

pub mod errors {
    include!(concat!(env!("OUT_DIR"), "/errors.rs"));

    /// `text` with `&1`, `&2`, ... replaced by the values of the same
    /// position. A position with no value stays as written.
    pub fn substitute<V: AsRef<[u8]>>(text: &str, values: &[V]) -> Vec<u8> {
        let mut out = Vec::with_capacity(text.len());
        let mut bytes = text.as_bytes().iter().copied().peekable();
        while let Some(byte) = bytes.next() {
            if byte != b'&' {
                out.push(byte);
                continue;
            }
            match bytes.peek().copied() {
                Some(digit @ b'1'..=b'9') => {
                    bytes.next();
                    match values.get(usize::from(digit - b'1')) {
                        Some(value) => out.extend_from_slice(value.as_ref()),
                        None => out.extend_from_slice(&[b'&', digit]),
                    }
                }
                _ => out.push(b'&'),
            }
        }
        out
    }
}

pub mod builtins {
    include!(concat!(env!("OUT_DIR"), "/builtins.rs"));

    /// The builtins excluded, whole or in part, each with its owner and the
    /// reason: the whole exclusions followed by the partial rows.
    pub const EXCLUDED: &[(&str, &str, &str)] = &[];

    /// The owner of `name`'s excluded form and its reason, or `None` for
    /// a name [`EXCLUDED`] does not list.
    pub fn owner_of(name: &str) -> Option<(&'static str, &'static str)> {
        EXCLUDED
            .iter()
            .find(|(excluded, _, _)| *excluded == name)
            .map(|(_, owner, reason)| (*owner, *reason))
    }

    /// Every excluded name, in [`EXCLUDED`] order.
    pub fn excluded_names() -> Vec<&'static str> {
        EXCLUDED.iter().map(|(name, _, _)| *name).collect()
    }

    /// The subset of [`EXCLUDED`] that is excluded only in part, and so is
    /// still in scope in its other form. Named rather than written as a
    /// literal `3` at the two places that subtract it, because "in
    /// `EXCLUDED`" and "excluded outright" are different sets and the
    /// difference is exactly these names.
    pub const PARTIALLY_EXCLUDED: &[&str] = &[];

    /// The builtins excluded outright: [`EXCLUDED`] less
    /// [`PARTIALLY_EXCLUDED`], in `EXCLUDED` order.
    pub fn wholly_excluded() -> Vec<&'static str> {
        EXCLUDED
            .iter()
            .map(|(name, _, _)| *name)
            .filter(|name| !PARTIALLY_EXCLUDED.contains(name))
            .collect()
    }

    /// The builtins this crate must be able to run: every name in [`NAMES`] that
    /// is not excluded outright, in `NAMES` order. A partially excluded name
    /// is in here, because its in-scope form still has to work.
    pub fn in_scope() -> Vec<&'static str> {
        let whole = wholly_excluded();
        NAMES
            .iter()
            .copied()
            .filter(|name| !whole.contains(name))
            .collect()
    }
}
