//! The seed of an interpreter's keyed hash of strings.

use siphasher::sip::SipHasher13;

/// The key of the keyed hash an interpreter hashes strings with, a string's
/// hash code (`r[class.string.hashcode]`), and of a stem's map from tails to
/// ordinals, which hashes with a function of its own ([`HashSeed::four_keys`]).
/// Keys that collide for one seed do not in general collide for
/// another, so a program cannot choose a set of strings that all land on one
/// probe path without knowing the seed.
///
/// There is no default: every holder is given its interpreter's seed, so two
/// tables of one interpreter hash alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashSeed {
    k0: u64,
    k1: u64,
}

impl HashSeed {
    /// The seed with the key `k0`, `k1`.
    pub const fn new(k0: u64, k1: u64) -> HashSeed {
        HashSeed { k0, k1 }
    }

    /// The seed a single number names, spread over the whole key, so that
    /// nearby numbers give unrelated keys.
    pub fn from_u64(seed: u64) -> HashSeed {
        let mut state = seed;
        HashSeed::new(splitmix64(&mut state), splitmix64(&mut state))
    }

    /// A seed drawn from the process's source of randomness, a different one
    /// at each call.
    pub fn random() -> HashSeed {
        use std::hash::BuildHasher;
        let state = std::hash::RandomState::new();
        HashSeed::new(state.hash_one(0u8), state.hash_one(1u8))
    }

    /// Four keys derived from this seed, for a keyed hash that takes four:
    /// the seed's own two, and two mixed from both of them.
    pub(crate) fn four_keys(self) -> [u64; 4] {
        let mut state = self.k0 ^ self.k1.rotate_left(32);
        [
            self.k0,
            self.k1,
            splitmix64(&mut state),
            splitmix64(&mut state),
        ]
    }

    /// The keyed hash of `bytes`.
    #[inline]
    pub fn hash(self, bytes: &[u8]) -> u64 {
        SipHasher13::new_with_keys(self.k0, self.k1).hash(bytes)
    }
}

impl std::hash::BuildHasher for HashSeed {
    type Hasher = SipHasher13;

    #[inline]
    fn build_hasher(&self) -> SipHasher13 {
        SipHasher13::new_with_keys(self.k0, self.k1)
    }
}

/// One splitmix64 step: advances `state` and answers the mixed output.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_seeds_hash_one_string_differently() {
        assert_ne!(
            HashSeed::from_u64(1).hash(b"abc"),
            HashSeed::from_u64(2).hash(b"abc")
        );
        assert_eq!(
            HashSeed::from_u64(1).hash(b"abc"),
            HashSeed::from_u64(1).hash(b"abc")
        );
    }

    #[test]
    fn random_seeds_differ() {
        assert_ne!(HashSeed::random(), HashSeed::random());
    }
}
