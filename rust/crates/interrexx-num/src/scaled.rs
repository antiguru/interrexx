//! A short number as a scaled machine integer, for the operations a
//! controlled loop repeats on every pass.
//!
//! Every operation here answers exactly what the [`Number`] operation it
//! stands for answers, or `None` where it cannot be sure to; the caller then
//! takes the `Number` route. That is the whole contract: a `Scaled` is never
//! a second arithmetic, only a cheaper spelling of the cases where the first
//! one is exact.

use std::cmp::Ordering;

use crate::{CompareOp, Digits, MAX_EXPONENT, MIN_EXPONENT, Number};

/// The widest mantissa held: eighteen decimal digits always fit an `i64`,
/// with room left for the sum of two of them.
const WIDEST: u32 = 18;

/// `mantissa * 10^exponent`, with the [`Number`]'s own digit count: a
/// trailing zero is a digit here as it is there (`1.10` is `110` at `-2`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scaled {
    mantissa: i64,
    exponent: i32,
    /// Digits in `mantissa`, `1` for zero.
    length: u32,
}

impl Number {
    /// This number as a [`Scaled`], when its digits fit one.
    pub fn scaled(&self) -> Option<Scaled> {
        let length = self.digits.len() as u32;
        if length > WIDEST {
            return None;
        }
        let magnitude = self
            .digits
            .iter()
            .fold(0i64, |acc, digit| acc * 10 + i64::from(*digit));
        Some(Scaled {
            mantissa: if self.negative { -magnitude } else { magnitude },
            exponent: self.exponent,
            length,
        })
    }
}

/// `10^n` for every `n` a [`WIDEST`]-digit value can be shifted by.
const POWERS: [i64; WIDEST as usize + 1] = {
    let mut powers = [1i64; WIDEST as usize + 1];
    let mut n = 1;
    while n < powers.len() {
        powers[n] = powers[n - 1] * 10;
        n += 1;
    }
    powers
};

/// How many decimal digits `magnitude` has, `1` for zero; `magnitude` is
/// below `10^WIDEST`.
fn digit_length(magnitude: u64) -> u32 {
    let mut length = 1;
    while length < WIDEST && magnitude >= POWERS[length as usize] as u64 {
        length += 1;
    }
    length
}

/// `value * 10^shift`, where the caller has bounded the result to [`WIDEST`]
/// digits.
fn shifted(value: i64, shift: u32) -> i64 {
    value * POWERS[shift as usize]
}

impl Scaled {
    /// The [`Number`] this spells.
    pub fn to_number(self) -> Number {
        if self.mantissa == 0 {
            return Number::zero();
        }
        let mut magnitude = self.mantissa.unsigned_abs();
        let length = self.length as usize;
        let mut digits = Digits::zeros(length);
        let out = &mut digits[..];
        for slot in out.iter_mut().rev() {
            *slot = (magnitude % 10) as u8;
            magnitude /= 10;
        }
        Number {
            negative: self.mantissa < 0,
            digits,
            exponent: self.exponent,
        }
    }

    /// `value` as a `Scaled`, as [`Number::from_i64`] spells it.
    pub fn from_i64(value: i64) -> Option<Scaled> {
        let magnitude = value.unsigned_abs();
        if magnitude >= POWERS[WIDEST as usize] as u64 {
            return None;
        }
        Some(Scaled {
            mantissa: value,
            exponent: 0,
            length: digit_length(magnitude),
        })
    }

    /// [`crate::compare_numbers`] with `FUZZ` 0 for a non-strict `op`, where
    /// [`Scaled::order`] answers.
    pub fn compare(self, other: Scaled, digits: u64, op: CompareOp) -> Option<bool> {
        Some(op.holds(self.order(other, digits)?))
    }

    /// The value with its sign turned, which is how [`Number::sub`] reads
    /// its right operand.
    pub fn negated(self) -> Scaled {
        Scaled {
            mantissa: -self.mantissa,
            ..self
        }
    }

    /// How many digits the value has, as [`Number::digit_count`] counts them.
    pub fn digit_count(self) -> u32 {
        self.length
    }

    /// `(mantissa, exponent)`, for a caller that builds a handle from them.
    pub fn parts(self) -> (i64, i32) {
        (self.mantissa, self.exponent)
    }

    pub fn is_negative(self) -> bool {
        self.mantissa < 0
    }

    /// [`Number::add`] at `digits`, where the sum needs no rounding and no
    /// operand is shortened; `None` otherwise.
    pub fn add(self, other: Scaled, digits: u64) -> Option<Scaled> {
        // An operand longer than DIGITS is shortened before the addition, and
        // `add`'s zero shortcut rounds the other operand: both are excluded.
        if u64::from(self.length) > digits || u64::from(other.length) > digits {
            return None;
        }
        // `add`'s own shortcuts for a zero operand answer the other one as it
        // stands, its exponent included.
        if self.mantissa == 0 {
            return Some(other);
        }
        if other.mantissa == 0 {
            return Some(self);
        }
        // `add`'s exact sum, under the bounds it takes it under: the aligned
        // width plus a carry digit within both DIGITS and an `i64`. Outside
        // them it aligns and rounds in ways a plain sum does not reproduce.
        let longest = self.length.max(other.length);
        if longest >= WIDEST || u64::from(longest) >= digits {
            return None;
        }
        let exponent = self.exponent.min(other.exponent);
        let left_shift = u32::try_from(i64::from(self.exponent) - i64::from(exponent)).ok()?;
        let right_shift = u32::try_from(i64::from(other.exponent) - i64::from(exponent)).ok()?;
        let width = (self.length + left_shift).max(other.length + right_shift) + 1;
        if width > WIDEST || u64::from(width) > digits {
            return None;
        }
        let sum = shifted(self.mantissa, left_shift) + shifted(other.mantissa, right_shift);
        if sum == 0 {
            return Some(Scaled {
                mantissa: 0,
                exponent: 0,
                length: 1,
            });
        }
        let result = Scaled {
            mantissa: sum,
            exponent,
            length: digit_length(sum.unsigned_abs()),
        };
        // `add`'s range check, which only a value at the edge of the
        // exponent range can fail.
        let adjusted = i64::from(exponent) + i64::from(result.length) - 1;
        if adjusted > i64::from(MAX_EXPONENT) || exponent < MIN_EXPONENT {
            return None;
        }
        Some(result)
    }

    /// [`Number::mul`] at `digits`, where the product needs no rounding and
    /// no operand is shortened; `None` otherwise. The product keeps every
    /// digit, trailing zeros included, as the long multiplication does.
    pub fn mul(self, other: Scaled, digits: u64) -> Option<Scaled> {
        if u64::from(self.length) > digits || u64::from(other.length) > digits {
            return None;
        }
        if self.mantissa == 0 || other.mantissa == 0 {
            return Some(Scaled {
                mantissa: 0,
                exponent: 0,
                length: 1,
            });
        }
        // Eighteen digits between them bound the product to an `i64`.
        if self.length + other.length > WIDEST {
            return None;
        }
        let product = self.mantissa * other.mantissa;
        let length = digit_length(product.unsigned_abs());
        if u64::from(length) > digits {
            return None;
        }
        let exponent = self.exponent.checked_add(other.exponent)?;
        let adjusted = i64::from(exponent) + i64::from(length) - 1;
        if adjusted > i64::from(MAX_EXPONENT) || exponent < MIN_EXPONENT {
            return None;
        }
        Some(Scaled {
            mantissa: product,
            exponent,
            length,
        })
    }

    /// The numeric comparison at `digits` with `FUZZ` 0, where both values
    /// align within DIGITS digits and so compare exactly; `None` otherwise.
    pub fn order(self, other: Scaled, digits: u64) -> Option<Ordering> {
        let exponent = self.exponent.min(other.exponent);
        let left_shift = u32::try_from(i64::from(self.exponent) - i64::from(exponent)).ok()?;
        let right_shift = u32::try_from(i64::from(other.exponent) - i64::from(exponent)).ok()?;
        let left_width = self.length + left_shift;
        let right_width = other.length + right_shift;
        // Past DIGITS the comparison is a subtraction that rounds, and it
        // can call two different values equal.
        let limit = u64::from(left_width.max(right_width));
        if limit > digits || limit > u64::from(WIDEST) {
            return None;
        }
        Some(shifted(self.mantissa, left_shift).cmp(&shifted(other.mantissa, right_shift)))
    }

    /// [`Number::write_short_fraction`] of the number this spells.
    pub fn write_short_fraction(self, digits: u64, out: &mut [u8]) -> Option<usize> {
        if self.exponent >= 0 || u64::from(self.length) > digits {
            return None;
        }
        let count = self.length as usize;
        let places = self.exponent.unsigned_abs() as usize;
        let adjusted = i64::from(self.exponent) + count as i64 - 1;
        if adjusted < 0 && places as u64 > digits.saturating_mul(2) {
            return None;
        }
        let negative = self.mantissa < 0;
        let whole = count.saturating_sub(places);
        let length = usize::from(negative) + whole.max(1) + 1 + places;
        if length > out.len() {
            return None;
        }
        // Written from the right: the fraction's digits, the zeros between
        // them and the point, the point, then the whole part or its `0`.
        let mut magnitude = self.mantissa.unsigned_abs();
        let mut at = length;
        for _ in 0..places {
            at -= 1;
            out[at] = b'0' + (magnitude % 10) as u8;
            magnitude /= 10;
        }
        at -= 1;
        out[at] = b'.';
        if whole == 0 {
            at -= 1;
            out[at] = b'0';
        } else {
            for _ in 0..whole {
                at -= 1;
                out[at] = b'0' + (magnitude % 10) as u8;
                magnitude /= 10;
            }
        }
        if negative {
            at -= 1;
            out[at] = b'-';
        }
        debug_assert_eq!(at, 0);
        Some(length)
    }
}
