//! The hasher behind [`crate::NameMap`] on a target whose pointers are
//! narrower than 64 bits: `rustc-hash` 2.1's `FxHasher` as it computes on a
//! 64-bit target, ported from that crate (MIT OR Apache-2.0).
//!
//! `rustc-hash` hashes with the target's pointer width, so on a 32-bit target
//! it answers other hashes for the same names. The standard map places an
//! entry by the low bits of its hash, so other hashes give the map another
//! layout, and a map's iteration order, which `Class~methods` answers in,
//! would then differ from the 64-bit build's. With this hasher a name hashes
//! alike on every target, and a map of names is laid out alike wherever the
//! map's probe groups are as wide.

use interrexx_cast::{CastFrom, CastTruncate};
use std::hash::{BuildHasher, Hasher};

/// The multiplier of the 64-bit hash.
const K: u64 = 0xf135_7aea_2e62_a9c5;

const SEED1: u64 = 0x243f_6a88_85a3_08d3;
const SEED2: u64 = 0x1319_8a2e_0370_7344;
const PREVENT_TRIVIAL_ZERO_COLLAPSE: u64 = 0xa409_3822_299f_31d0;

/// Builds a [`WideFxHasher`].
#[derive(Clone, Copy, Debug, Default)]
pub struct WideFxBuildHasher;

impl BuildHasher for WideFxBuildHasher {
    type Hasher = WideFxHasher;

    #[inline]
    fn build_hasher(&self) -> WideFxHasher {
        WideFxHasher { hash: 0 }
    }
}

/// `rustc-hash`'s `FxHasher` with its 64-bit state, constant and rotation on
/// every target.
#[derive(Clone, Copy, Debug, Default)]
pub struct WideFxHasher {
    hash: u64,
}

impl WideFxHasher {
    #[inline]
    fn add_to_hash(&mut self, i: u64) {
        self.hash = self.hash.wrapping_add(i).wrapping_mul(K);
    }
}

impl Hasher for WideFxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.write_u64(hash_bytes(bytes));
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add_to_hash(u64::from(i));
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add_to_hash(u64::from(i));
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add_to_hash(u64::from(i));
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i);
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.add_to_hash(u64::cast_truncate(i));
        self.add_to_hash(u64::cast_truncate(i >> 64));
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add_to_hash(u64::cast_from(i));
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash.rotate_left(26)
    }
}

/// The 64-bit product of `x` and `y` folded to 64 bits, as a 64-bit target
/// computes it.
#[inline]
fn multiply_mix(x: u64, y: u64) -> u64 {
    let full = u128::from(x).wrapping_mul(u128::from(y));
    u64::cast_truncate(full) ^ u64::cast_truncate(full >> 64)
}

/// A byte string compressed to one word.
#[inline]
fn hash_bytes(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut s0 = SEED1;
    let mut s1 = SEED2;
    let word = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("8 bytes"));
    if len <= 16 {
        if len >= 8 {
            s0 ^= word(0);
            s1 ^= word(len - 8);
        } else if len >= 4 {
            let half = |at: usize| {
                u64::from(u32::from_le_bytes(
                    bytes[at..at + 4].try_into().expect("4 bytes"),
                ))
            };
            s0 ^= half(0);
            s1 ^= half(len - 4);
        } else if len > 0 {
            s0 ^= u64::from(bytes[0]);
            s1 ^= (u64::from(bytes[len - 1]) << 8) | u64::from(bytes[len / 2]);
        }
    } else {
        let mut bulk = &bytes[..len - 1];
        while let Some((chunk, rest)) = bulk.split_first_chunk::<16>() {
            let x = u64::from_le_bytes(chunk[..8].try_into().expect("8 bytes"));
            let y = u64::from_le_bytes(chunk[8..].try_into().expect("8 bytes"));
            let mixed = multiply_mix(s0 ^ x, PREVENT_TRIVIAL_ZERO_COLLAPSE ^ y);
            s0 = s1;
            s1 = mixed;
            bulk = rest;
        }
        s0 ^= word(len - 16);
        s1 ^= word(len - 8);
    }
    multiply_mix(s0, s1) ^ u64::cast_from(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hashes `rustc-hash` 2.1.3 answers on a 64-bit target, which this
    /// hasher answers on every target.
    #[test]
    fn names_hash_as_rustc_hash_does_on_a_64_bit_target() {
        let wide = WideFxBuildHasher;
        assert_eq!(wide.hash_one(""), 13_933_120_620_573_868_840);
        assert_eq!(wide.hash_one("FOO"), 14_053_790_281_315_571_566);
        assert_eq!(wide.hash_one("OBJECTNAME="), 7_561_263_186_884_217_229);
        assert_eq!(
            wide.hash_one("a name longer than sixteen bytes"),
            15_450_639_358_844_404_332
        );
        assert_eq!(wide.hash_one(1u8), 12_157_901_119_326_311_915);
        assert_eq!(
            wide.hash_one(0x0123_4567_89ab_cdef_u64),
            3_210_671_808_277_117_407
        );
    }

    /// On a 64-bit target the two agree on names of every length up to and
    /// past the bulk loop's chunks.
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn names_hash_as_rustc_hash_does_here() {
        let name: String = (0..70u8).map(|at| char::from(b'A' + at % 26)).collect();
        for len in 0..=name.len() {
            let name = &name[..len];
            assert_eq!(
                WideFxBuildHasher.hash_one(name),
                rustc_hash::FxBuildHasher.hash_one(name),
                "{name:?}"
            );
        }
    }
}
