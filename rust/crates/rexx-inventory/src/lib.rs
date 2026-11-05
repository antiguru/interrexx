//! Tables mechanically derived from the C++ tree, plus the one hand-written
//! list that indexes into them.

pub mod errors {
    include!(concat!(env!("OUT_DIR"), "/errors.rs"));
}

pub mod builtins {
    include!(concat!(env!("OUT_DIR"), "/builtins.rs"));

    /// The builtins excluded, whole or in part, each with the phase that owes
    /// it and the reason: the whole exclusions followed by the partial rows.
    /// The owner rides here, so a name and the phase blamed for it cannot
    /// drift apart.
    pub const EXCLUDED: &[(&str, &str, &str)] = &[];

    /// The phase owing `name`'s excluded form and its reason, or `None` for
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
