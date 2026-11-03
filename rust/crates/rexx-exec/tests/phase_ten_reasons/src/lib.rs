// A fixture for `phase_ten_reasons.rs`'s negative control, walked as a crate's `src/` and never
// compiled: Phase 10 is named only in test modules, inline and in a file, which the production
// walk leaves out.
fn production() -> &'static str {
    "a production text"
}

#[cfg(test)]
mod inline {
    const OWNER: &str = "Phase 10";
}

#[cfg(test)]
mod tests;
