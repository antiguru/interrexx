//! SHA-256, for the pins on the embedded sources. `build.rs` compiles this
//! file through `#[path]` and the crate's own tests use it, so the digest is
//! taken one way.

use sha2::{Digest, Sha256};

/// SHA-256 of `bytes`, lower-case hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
