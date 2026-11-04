// A fixture for `closed_phases.rs`'s negative control, walked as a crate's `src/` and never
// compiled: refusals naming a phase that is still open, alone and inside a longer text.
fn refuse() -> Loud {
    Loud::new("the instance API is not implemented (Phase 11)", "Phase 11")
}
