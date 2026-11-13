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

/// `product` (nonzero) rounded to its first `digits` digits, half up on
/// the first digit dropped, an all-nines carry shedding a digit: the
/// magnitude kept and how many places it was moved, or `None` where the
/// kept digits are more than [`WIDEST`].
#[inline]
fn rounded(product: u128, digits: u64) -> Option<(u64, i32)> {
    let length = match u64::try_from(product) {
        Ok(narrow) => narrow.ilog10() + 1,
        Err(_) => product.ilog10() + 1,
    };
    if u64::from(length) <= digits {
        return (length <= WIDEST).then_some((product as u64, 0));
    }
    // DIGITS is below `length`, which is at most thirty-eight.
    let keep = digits as u32;
    if keep > WIDEST {
        return None;
    }
    let dropped = length - keep;
    let (mut kept, first_dropped) = if product <= u128::from(u64::MAX) {
        // The common case, without a 128-bit division.
        let product = product as u64;
        let unit = 10u64.pow(dropped - 1);
        let tail = product / unit;
        (tail / 10, tail % 10)
    } else {
        let unit = 10u128.pow(dropped - 1);
        let tail = product / unit;
        ((tail / 10) as u64, (tail % 10) as u64)
    };
    let mut moved = dropped as i32;
    if first_dropped >= 5 {
        kept += 1;
        if kept == POWERS[keep as usize] as u64 {
            kept /= 10;
            moved += 1;
        }
    }
    Some((kept, moved))
}

impl Scaled {
    const ZERO: Scaled = Scaled {
        mantissa: 0,
        exponent: 0,
        length: 1,
    };

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

    /// `bytes` as a `Scaled` where they are a plain nonzero decimal -- an
    /// optional `-`, digits and at most one point, nothing else, no more
    /// than [`WIDEST`] digits -- which is [`Number::parse_bytes`]'s answer
    /// read without a [`Number`] built; `None` for every other spelling,
    /// numeric or not.
    pub fn parse_plain(bytes: &[u8]) -> Option<Scaled> {
        let (negative, body) = match bytes.split_first() {
            Some((b'-', rest)) => (true, rest),
            _ => (false, bytes),
        };
        let mut magnitude = 0i64;
        let mut count = 0u32;
        let mut decimals = 0i32;
        let mut seen_point = false;
        for &byte in body {
            match byte {
                b'0'..=b'9' => {
                    count += 1;
                    if count > WIDEST {
                        return None;
                    }
                    magnitude = magnitude * 10 + i64::from(byte - b'0');
                    decimals += i32::from(seen_point);
                }
                b'.' if !seen_point => seen_point = true,
                _ => return None,
            }
        }
        if magnitude == 0 {
            // No digit at all, or a zero, which `parse_bytes` spells its
            // own way.
            return None;
        }
        Some(Scaled {
            mantissa: if negative { -magnitude } else { magnitude },
            exponent: -decimals,
            length: digit_length(magnitude as u64),
        })
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

    /// [`Number::add`] at `digits`, where no operand is shortened before
    /// the addition and the aligned operands fit [`WIDEST`] digits; `None`
    /// otherwise. Within those bounds this is the addition's general path
    /// on machine integers: its fast path for an operand too far below the
    /// other to reach the result, its alignment adjustment, and its
    /// rounding of the raw sum, leading zeros counted.
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
        // **The exact sum first**: the aligned width plus a carry digit
        // within both DIGITS and [`WIDEST`], where nothing is adjusted or
        // rounded -- the case a loop's own counter is in on every pass.
        let exponent = self.exponent.min(other.exponent);
        let left_shift = self.exponent.abs_diff(exponent);
        let right_shift = other.exponent.abs_diff(exponent);
        let width = (self.length.saturating_add(left_shift))
            .max(other.length.saturating_add(right_shift))
            .saturating_add(1);
        if width <= WIDEST && u64::from(width) <= digits {
            let sum = shifted(self.mantissa, left_shift) + shifted(other.mantissa, right_shift);
            if sum == 0 {
                return Some(Scaled::ZERO);
            }
            let result = Scaled {
                mantissa: sum,
                exponent,
                length: digit_length(sum.unsigned_abs()),
            };
            let adjusted = i64::from(exponent) + i64::from(result.length) - 1;
            if adjusted > i64::from(MAX_EXPONENT) || exponent < MIN_EXPONENT {
                return None;
            }
            return Some(result);
        }
        self.add_rounded(other, digits)
    }

    /// [`Scaled::add`] where the sum is not plainly exact.
    #[inline(never)]
    fn add_rounded(self, other: Scaled, digits: u64) -> Option<Scaled> {
        let first_min = i64::from(self.exponent.min(other.exponent));
        let left_shift = i64::from(self.exponent) - first_min;
        let right_shift = i64::from(other.exponent) - first_min;
        let (left_length, right_length) = (i64::from(self.length), i64::from(other.length));
        // The operand the other is too far below to reach any kept digit,
        // answered as it stands (it is no longer than DIGITS, so its own
        // rounding leaves it alone). Saturated: DIGITS can be past `i64`.
        let reach = i64::try_from(digits).unwrap_or(i64::MAX);
        if left_shift + left_length > right_length.saturating_add(reach) {
            return Some(self);
        }
        if right_shift + right_length > left_length.saturating_add(reach) {
            return Some(other);
        }
        // The alignment adjustment, as `aligned_wide_sum` takes it: where
        // the pair spans more than the working length, the less significant
        // operand loses low digits first and then both do, never emptied,
        // with the whole count folded into the exponent.
        let max_length = reach.saturating_add(1);
        let (mut left, mut right) = (self, other);
        let left_over = left_length + left_shift - max_length;
        let right_over = right_length + right_shift - max_length;
        if left_over > 0 || right_over > 0 {
            let mut adjust = left_over.max(right_over);
            if left_shift != 0 {
                let taken = adjust.min(left_shift);
                right = right.shortened(taken)?;
                adjust -= taken;
            } else if right_shift != 0 {
                let taken = adjust.min(right_shift);
                left = left.shortened(taken)?;
                adjust -= taken;
            }
            if adjust != 0 {
                left = left.shortened(adjust)?;
                right = right.shortened(adjust)?;
            }
        }
        let exponent = left.exponent.min(right.exponent);
        let left_shift = u32::try_from(i64::from(left.exponent) - i64::from(exponent)).ok()?;
        let right_shift = u32::try_from(i64::from(right.exponent) - i64::from(exponent)).ok()?;
        let widest = (left.length + left_shift).max(right.length + right_shift);
        if widest > WIDEST || i64::from(widest) > max_length {
            return None;
        }
        let a = shifted(left.mantissa.abs(), left_shift);
        let b = shifted(right.mantissa.abs(), right_shift);
        let (raw, negative, raw_length) = if left.is_negative() == right.is_negative() {
            let sum = a + b;
            let carried = sum >= POWERS[widest as usize];
            (sum, left.is_negative(), widest + u32::from(carried))
        } else {
            match a.cmp(&b) {
                Ordering::Equal => return Some(Scaled::ZERO),
                Ordering::Greater => (a - b, left.is_negative(), widest),
                Ordering::Less => (b - a, right.is_negative(), widest),
            }
        };
        // `round_change` on the raw digits: the first DIGITS of them kept,
        // leading zeros counted, half up on the first dropped digit, and an
        // all-nines carry keeps the length and moves the exponent.
        let (mut magnitude, mut exponent) = (raw, exponent);
        if u64::from(raw_length) > digits {
            // `raw_length` is at most `WIDEST + 1` and DIGITS is below it.
            let keep = digits as u32;
            let dropped = raw_length - keep;
            exponent = exponent.checked_add(dropped as i32)?;
            let unit = 10i64.pow(dropped);
            let round_up = (magnitude / (unit / 10)) % 10 >= 5;
            magnitude /= unit;
            if round_up {
                magnitude += 1;
                if magnitude == 10i64.pow(keep) {
                    magnitude /= 10;
                    exponent = exponent.checked_add(1)?;
                }
            }
        }
        if magnitude == 0 {
            return Some(Scaled::ZERO);
        }
        // A sum of nineteen digits that DIGITS keeps whole.
        if magnitude >= POWERS[WIDEST as usize] {
            return None;
        }
        let result = Scaled {
            mantissa: if negative { -magnitude } else { magnitude },
            exponent,
            length: digit_length(magnitude as u64),
        };
        // `add`'s range check, which only a value at the edge of the
        // exponent range can fail.
        let adjusted = i64::from(exponent) + i64::from(result.length) - 1;
        if adjusted > i64::from(MAX_EXPONENT) || exponent < MIN_EXPONENT {
            return None;
        }
        Some(result)
    }

    /// The value with `count` low digits dropped (truncated, at least one
    /// kept) and `count` added to its exponent whatever was dropped, as the
    /// addition's alignment adjustment shortens an operand.
    fn shortened(self, count: i64) -> Option<Scaled> {
        let length = i64::from(self.length);
        let kept = (length - count.max(0)).max(1);
        let dropped = (length - kept) as usize;
        Some(Scaled {
            mantissa: self.mantissa / POWERS[dropped],
            exponent: self.exponent.checked_add(i32::try_from(count).ok()?)?,
            length: kept as u32,
        })
    }

    /// [`Number::mul`] at `digits`, where no operand is shortened and the
    /// rounded product fits [`WIDEST`] digits; `None` otherwise. The product
    /// keeps every digit DIGITS allows, trailing zeros included, and the
    /// rest round half up on the first one dropped, as the long
    /// multiplication's rounding does.
    pub fn mul(self, other: Scaled, digits: u64) -> Option<Scaled> {
        if u64::from(self.length) > digits || u64::from(other.length) > digits {
            return None;
        }
        if self.mantissa == 0 || other.mantissa == 0 {
            return Some(Scaled::ZERO);
        }
        let (a, b) = (self.mantissa.unsigned_abs(), other.mantissa.unsigned_abs());
        // A product that fits a `u64` is rounded there; a wider one, of two
        // operands of up to eighteen digits each, in a `u128`.
        let (magnitude, dropped) = match a.checked_mul(b) {
            Some(product) => rounded(u128::from(product), digits)?,
            None => rounded(u128::from(a) * u128::from(b), digits)?,
        };
        let exponent = self
            .exponent
            .checked_add(other.exponent)?
            .checked_add(dropped)?;
        let length = digit_length(magnitude);
        let adjusted = i64::from(exponent) + i64::from(length) - 1;
        if adjusted > i64::from(MAX_EXPONENT) || exponent < MIN_EXPONENT {
            return None;
        }
        let mantissa = magnitude as i64;
        Some(Scaled {
            mantissa: if self.is_negative() != other.is_negative() {
                -mantissa
            } else {
                mantissa
            },
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

#[cfg(test)]
mod tests {
    use super::Scaled;
    use crate::Number;

    /// `Scaled::parse_plain` answers what `Number::parse_bytes` answers
    /// wherever it answers, and answers every plain short decimal.
    #[test]
    fn the_plain_parse_answers_what_the_number_parse_answers() {
        let alphabet = b"0123456789.-+ eE";
        let mut state = 0x1234_5678_9abc_def1u64;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        let mut taken = 0;
        for _ in 0..200_000 {
            let length = next(22) as usize;
            let bytes: Vec<u8> = (0..length)
                .map(|_| {
                    // Mostly digits, so that plain spellings are common.
                    if next(3) == 0 {
                        alphabet[next(alphabet.len() as u64) as usize]
                    } else {
                        b'0' + next(10) as u8
                    }
                })
                .collect();
            let expected = Number::parse_bytes(&bytes);
            match Scaled::parse_plain(&bytes) {
                Some(scaled) => {
                    taken += 1;
                    assert_eq!(
                        Some(scaled.to_number()),
                        expected,
                        "{}",
                        String::from_utf8_lossy(&bytes)
                    );
                    assert_eq!(expected.as_ref().and_then(Number::scaled), Some(scaled));
                }
                None => {
                    let plain = bytes.iter().filter(|b| b.is_ascii_digit()).count() <= 18
                        && bytes.iter().all(|b| b.is_ascii_digit() || *b == b'.')
                        && bytes.iter().filter(|b| **b == b'.').count() <= 1;
                    assert!(
                        !plain || expected.as_ref().is_none_or(Number::is_zero),
                        "{} declined",
                        String::from_utf8_lossy(&bytes)
                    );
                }
            }
        }
        assert!(taken > 50_000, "the plain parse answered {taken} times");
    }

    /// `Scaled::mul` answers what the long multiplication answers wherever
    /// it answers at all, rounding included.
    #[test]
    fn the_scaled_product_answers_what_the_long_multiplication_answers() {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        let number = |next: &mut dyn FnMut(u64) -> u64| {
            let length = 1 + next(18) as usize;
            let mut text = String::new();
            if next(2) == 0 {
                text.push('-');
            }
            for i in 0..length {
                // Runs of nines, which is where an all-nines carry lives.
                let digit = match next(3) {
                    0 => 9,
                    1 => 0,
                    _ => next(10) as u8,
                };
                text.push(char::from(b'0' + if i == 0 { digit.max(1) } else { digit }));
            }
            text.push_str(&format!("E{}", next(20) as i64 - 10));
            if next(16) == 0 {
                text = String::from("0");
            }
            Number::parse(&text).expect("a generated spelling parses")
        };
        let (mut taken, mut rounded, mut wide) = (0, 0, 0);
        for _ in 0..200_000 {
            let left = number(&mut next);
            let right = number(&mut next);
            let digits = 1 + next(20);
            let (Some(l), Some(r)) = (left.scaled(), right.scaled()) else {
                continue;
            };
            let Some(product) = l.mul(r, digits) else {
                continue;
            };
            let expected = left
                .mul(&right, digits)
                .expect("a product the scaled path answers is in range");
            assert_eq!(
                product.to_number(),
                expected,
                "{} * {} at DIGITS {digits}",
                left.format(40),
                right.format(40)
            );
            taken += 1;
            let width = left.digits.len() + right.digits.len();
            if width as u64 > digits + 1 {
                rounded += 1;
            }
            if width > 19 {
                wide += 1;
            }
        }
        assert!(taken > 50_000, "the scaled product answered {taken} times");
        assert!(
            rounded > 20_000,
            "the scaled product rounded {rounded} times"
        );
        assert!(wide > 5_000, "the scaled product went wide {wide} times");
    }
}
