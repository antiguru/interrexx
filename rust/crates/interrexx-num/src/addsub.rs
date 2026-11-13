//! Addition and subtraction.
//! ```text
//! DIGITS 9:   1e9 - 1  ->  1.00000000E+9    not 999999999
//! DIGITS 10:  1e9 - 1  ->  999999999
//! ```

use std::borrow::Cow;

use crate::{ArithError, Digits, Number};

impl Number {
    /// Extends the digit vector downward so both operands share an exponent.
    #[inline(always)]
    fn aligned_to(&self, exponent: i32) -> Digits {
        let pad = (self.exponent - exponent).max(0) as usize;
        let mut digits = self.digits.clone();
        digits.extend_zeros(pad);
        digits
    }

    #[inline]
    pub fn add(&self, other: &Number, digits: u64) -> Result<Number, ArithError> {
        self.add_signed(other, false, digits)
    }

    /// Prefix `+`: `0 + self`. A value no longer than `digits` is its own
    /// answer, range check aside, and is handed back without being copied.
    #[inline]
    pub fn plus(self, digits: u64) -> Result<Number, ArithError> {
        if self.digits.len() as u64 <= digits {
            self.check_range()?;
            return Ok(self);
        }
        Number::zero().add(&self, digits)
    }

    #[inline]
    pub fn sub(&self, other: &Number, digits: u64) -> Result<Number, ArithError> {
        self.add_signed(other, true, digits)
    }

    fn add_signed(
        &self,
        other: &Number,
        negate_other: bool,
        digits: u64,
    ) -> Result<Number, ArithError> {
        let left_negative = self.negative;
        let right_negative = other.negative != negate_other;

        // Operands longer than DIGITS are truncated to DIGITS + 1 working
        // digits first. (The interpreter also raises LOSTDIGITS here.)
        let max_length = crate::working_length(digits);
        let left = self.truncated_to(max_length);
        let right = other.truncated_to(max_length);

        let min_exp = left.exponent.min(right.exponent);
        let adjusted_left_exp = (left.exponent - min_exp) as usize;
        let adjusted_right_exp = (right.exponent - min_exp) as usize;
        let left_len = left.digits.len();
        let right_len = right.digits.len();
        // Saturated for the fast-path sums below: `digits` is a bare u64,
        // and `len + digits_usize` must compare large, not wrap.
        let digits_usize = usize::try_from(digits).unwrap_or(usize::MAX);

        // Fast paths. Each returns one operand essentially untouched, and they
        // are not an optimisation: going through the general path produces a
        // leading zero that rounding then turns into a different number. `0 +
        // 123456789` is 123456789, but computing it yields `0123456789`, which
        // rounds to 123456790.
        let fast = if left.is_zero() {
            Some((&right, right_negative))
        } else if right.is_zero() {
            Some((&left, left_negative))
        } else if adjusted_left_exp + left_len > right_len.saturating_add(digits_usize) {
            // The right number is too far below the left to reach any digit
            // the result will keep. (`saturating_add`: the saturated
            // `digits_usize` must stay "huge", not wrap back around small.)
            Some((&left, left_negative))
        } else if adjusted_right_exp + right_len > left_len.saturating_add(digits_usize) {
            Some((&right, right_negative))
        } else {
            None
        };
        if let Some((value, negative)) = fast {
            let mut result = value.round_to(digits).into_owned();
            result.negative = negative && !result.is_zero();
            result.check_range()?;
            return Ok(result);
        }

        if let Some(sum) = exact_small_sum(&left, &right, left_negative, right_negative, digits) {
            sum.check_range()?;
            return Ok(sum);
        }
        if let Some(sum) = aligned_wide_sum(&left, &right, left_negative, right_negative, digits) {
            sum.check_range()?;
            return Ok(sum);
        }
        // Alignment adjustment, ported from addSub. When the two operands
        // together span more than the working precision, the less significant
        // one is shortened from its low end so the pair fits. Skipping this
        // is not merely imprecise -- it changes which digit the rounding
        // decision sees, so `12.3400 - 9.999999995` at DIGITS 3 comes out as
        // 2.3 rather than 2.4.
        let mut left_digits = left.digits.clone();
        let mut right_digits = right.digits.clone();
        let mut left_exp = left.exponent;
        let mut right_exp = right.exponent;
        let adjusted_left_exp = adjusted_left_exp as i64;
        let adjusted_right_exp = adjusted_right_exp as i64;
        // Saturated: `max_length` came from the bare `digits` and may sit at
        // usize::MAX, which an `as i64` would fold to -1. i64::MAX keeps
        // both `adjusted_*_digits` differences negative, exactly as a
        // precision that large should -- nothing ever needs shortening.
        let max_len = i64::try_from(max_length).unwrap_or(i64::MAX);

        let adjusted_left_digits = left_digits.len() as i64 + adjusted_left_exp - max_len;
        let adjusted_right_digits = right_digits.len() as i64 + adjusted_right_exp - max_len;
        if adjusted_left_digits > 0 || adjusted_right_digits > 0 {
            let mut adjust = adjusted_left_digits.max(adjusted_right_digits);

            // Exactly one of the adjusted exponents is non-zero: the more
            // significant operand's. The other operand is the one shortened.
            if adjusted_left_exp != 0 {
                let taken = adjust.min(adjusted_left_exp);
                drop_low_digits(&mut right_digits, taken);
                right_exp = right_exp.saturating_add(taken as i32);
                adjust -= taken;
            } else if adjusted_right_exp != 0 {
                let taken = adjust.min(adjusted_right_exp);
                drop_low_digits(&mut left_digits, taken);
                left_exp = left_exp.saturating_add(taken as i32);
                adjust -= taken;
            }

            if adjust != 0 {
                drop_low_digits(&mut left_digits, adjust);
                left_exp = left_exp.saturating_add(adjust as i32);
                drop_low_digits(&mut right_digits, adjust);
                right_exp = right_exp.saturating_add(adjust as i32);
            }
        }

        let left = Number {
            negative: left_negative,
            digits: left_digits,
            exponent: left_exp,
        };
        let right = Number {
            negative: right_negative,
            digits: right_digits,
            exponent: right_exp,
        };
        let min_exp = left.exponent.min(right.exponent);
        let a = left.aligned_to(min_exp);
        let b = right.aligned_to(min_exp);

        let (raw_digits, negative) = if left_negative == right_negative {
            (add_magnitudes(&a, &b), left_negative)
        } else {
            match compare_magnitudes(&a, &b) {
                std::cmp::Ordering::Equal => return Ok(Number::zero()),
                std::cmp::Ordering::Greater => (sub_magnitudes(&a, &b), left_negative),
                std::cmp::Ordering::Less => (sub_magnitudes(&b, &a), right_negative),
            }
        };

        // Deliberately not `assemble` here: that strips leading zeros, and a
        // zero left by a borrow or an absent carry has to still be present
        // for `round_to` to count it. Normalising happens inside `round_to`,
        // after the rounding decision has been made.
        let raw = Number {
            negative,
            digits: raw_digits,
            exponent: min_exp,
        };
        let rounded = raw.into_round(digits);
        let result = Number::assemble(rounded.negative, rounded.digits, rounded.exponent);
        result.check_range()?;
        Ok(result)
    }

    /// Shortens an over-long operand to the working precision, as the addition
    /// and its operand check both do. Truncation, not rounding.
    pub(crate) fn truncated_to(&self, max_length: usize) -> Cow<'_, Number> {
        if self.digits.len() <= max_length {
            return Cow::Borrowed(self);
        }
        let dropped = self.digits.len() - max_length;
        Cow::Owned(Number {
            negative: self.negative,
            digits: Digits::from_slice(&self.digits[..max_length]),
            exponent: self.exponent + dropped as i32,
        })
    }
}

#[cfg(test)]
thread_local! {
    /// Turns [`exact_small_sum`] off for the running test thread, so that a
    /// pair it would answer can be put through the digit-vector addition and
    /// the two compared. Compiled out of every non-test build, which is why
    /// the shortcut can be tested against the path it replaces without that
    /// path being split into a function of its own -- measured, splitting it
    /// cost 18.9M instructions on `bench-programs/arith.rex`.
    static SHORTCUT_OFF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The exact sum, when both operands align into an `i64` mantissa and the
/// result needs no rounding at `digits`.
fn exact_small_sum(
    left: &Number,
    right: &Number,
    left_negative: bool,
    right_negative: bool,
    digits: u64,
) -> Option<Number> {
    #[cfg(test)]
    if SHORTCUT_OFF.with(std::cell::Cell::get) {
        return None;
    }
    // **The lengths alone decide most calls, and they decide them first.**
    // The width below is never less than the longer operand plus one, so an
    // operand already that long settles it without the exponents being read
    // at all -- which is the answer for every operand a program has widened
    // to its own precision, and for every one `div` hands back at the
    // inflated precision it works in.
    let longest = left.digits.len().max(right.digits.len()) as i64;
    if longest >= 18 || longest >= digits as i64 {
        return None;
    }
    let min_exp = i64::from(left.exponent.min(right.exponent));
    let left_shift = i64::from(left.exponent) - min_exp;
    let right_shift = i64::from(right.exponent) - min_exp;
    let width =
        (left.digits.len() as i64 + left_shift).max(right.digits.len() as i64 + right_shift) + 1;
    // Eighteen digits is the widest an `i64` holds whatever they are, and it
    // bounds the scaling loop below as well as the sum.
    if width > 18 || width > digits as i64 {
        return None;
    }
    let mantissa = |number: &Number, shift: i64, negative: bool| {
        let mut value = number
            .digits
            .iter()
            .fold(0i64, |acc, digit| acc * 10 + i64::from(*digit));
        for _ in 0..shift {
            value *= 10;
        }
        if negative { -value } else { value }
    };
    let sum =
        mantissa(left, left_shift, left_negative) + mantissa(right, right_shift, right_negative);
    if sum == 0 {
        return Some(Number::zero());
    }
    let mut result = Number::from_i64(sum);
    result.exponent = min_exp as i32;
    Some(result)
}

/// The widest aligned operand [`aligned_wide_sum`] takes: thirty-eight
/// digits always fit a `u128`, and so does the sum of two of them.
const WIDE_DIGITS: usize = 38;

/// The general addition below, on `u128` magnitudes, for operands that
/// after its alignment adjustment fit [`WIDE_DIGITS`] aligned to the lower
/// exponent. Within that bound the digit
/// vectors the general path builds are these integers written out, with
/// the raw length that path's rounding counts: the wider aligned operand,
/// plus one for a carry out of an addition, and with the leading zeros a
/// subtraction leaves.
fn aligned_wide_sum(
    left: &Number,
    right: &Number,
    left_negative: bool,
    right_negative: bool,
    digits: u64,
) -> Option<Number> {
    #[cfg(test)]
    if SHORTCUT_OFF.with(std::cell::Cell::get) {
        return None;
    }
    // The general path's alignment adjustment, on slices: where the pair
    // spans more than the working length, the less significant operand's
    // low digits go first and then both operands' -- truncated, never
    // emptied, with the count folded into the exponent whatever was left.
    let max_length = i64::try_from(crate::working_length(digits)).unwrap_or(i64::MAX);
    let (mut left_digits, mut left_exp) = (left.digits.as_slice(), left.exponent);
    let (mut right_digits, mut right_exp) = (right.digits.as_slice(), right.exponent);
    let first_min = left_exp.min(right_exp);
    let left_shift = i64::from(left_exp) - i64::from(first_min);
    let right_shift = i64::from(right_exp) - i64::from(first_min);
    let left_over = left_digits.len() as i64 + left_shift - max_length;
    let right_over = right_digits.len() as i64 + right_shift - max_length;
    if left_over > 0 || right_over > 0 {
        let shorten = |slice: &[u8], count: i64| -> usize {
            slice
                .len()
                .saturating_sub(usize::try_from(count.max(0)).unwrap_or(usize::MAX))
                .max(1)
        };
        let mut adjust = left_over.max(right_over);
        if left_shift != 0 {
            let taken = adjust.min(left_shift);
            right_digits = &right_digits[..shorten(right_digits, taken)];
            right_exp = right_exp.saturating_add(taken as i32);
            adjust -= taken;
        } else if right_shift != 0 {
            let taken = adjust.min(right_shift);
            left_digits = &left_digits[..shorten(left_digits, taken)];
            left_exp = left_exp.saturating_add(taken as i32);
            adjust -= taken;
        }
        if adjust != 0 {
            left_digits = &left_digits[..shorten(left_digits, adjust)];
            left_exp = left_exp.saturating_add(adjust as i32);
            right_digits = &right_digits[..shorten(right_digits, adjust)];
            right_exp = right_exp.saturating_add(adjust as i32);
        }
    }
    let min_exp = left_exp.min(right_exp);
    let left_width =
        left_digits.len() + usize::try_from(i64::from(left_exp) - i64::from(min_exp)).ok()?;
    let right_width =
        right_digits.len() + usize::try_from(i64::from(right_exp) - i64::from(min_exp)).ok()?;
    let widest = left_width.max(right_width);
    if widest > WIDE_DIGITS || widest as i64 > max_length {
        return None;
    }
    // Folded in a `u64` while nineteen digits fit one, which is every
    // operand of a DIGITS 18 or narrower addition.
    let magnitude = |digits: &[u8], width: usize| {
        let split = digits.len().min(19);
        let head = digits[..split]
            .iter()
            .fold(0u64, |acc, digit| acc * 10 + u64::from(*digit));
        let mut value = u128::from(head);
        for digit in &digits[split..] {
            value = value * 10 + u128::from(*digit);
        }
        value * 10u128.pow((width - digits.len()) as u32)
    };
    let a = magnitude(left_digits, left_width);
    let b = magnitude(right_digits, right_width);
    let (raw, negative, raw_length) = if left_negative == right_negative {
        let sum = a + b;
        let carried = sum >= 10u128.pow(widest as u32);
        (sum, left_negative, widest + usize::from(carried))
    } else {
        match a.cmp(&b) {
            std::cmp::Ordering::Equal => return Some(Number::zero()),
            std::cmp::Ordering::Greater => (a - b, left_negative, widest),
            std::cmp::Ordering::Less => (b - a, right_negative, widest),
        }
    };
    // The raw digits, leading zeros included, written out through one
    // division into two `u64` halves: a `u128` divided digit by digit is a
    // library call per digit.
    let mut buffer = [0u8; WIDE_DIGITS + 1];
    let raw_digits = &mut buffer[..raw_length];
    let split = 10u128.pow(19);
    let (mut high, mut low) = if raw < split {
        (0, raw as u64)
    } else {
        ((raw / split) as u64, (raw % split) as u64)
    };
    for (place, slot) in raw_digits.iter_mut().rev().enumerate() {
        if place < 19 {
            *slot = (low % 10) as u8;
            low /= 10;
        } else {
            *slot = (high % 10) as u8;
            high /= 10;
        }
    }
    // `round_change` on those digits: keep the first DIGITS of them, leading
    // zeros counted, half up on the first dropped digit, and an all-nines
    // carry keeps the length and moves the exponent.
    let keep = usize::try_from(digits).unwrap_or(usize::MAX);
    let mut exponent = min_exp;
    let kept = if keep != 0 && raw_length > keep {
        exponent += (raw_length - keep) as i32;
        let round_up = raw_digits[keep] >= 5;
        let kept = &mut raw_digits[..keep];
        if round_up {
            match kept.iter().rposition(|digit| *digit != 9) {
                Some(at) => {
                    kept[at] += 1;
                    kept[at + 1..].fill(0);
                }
                None => {
                    kept.fill(0);
                    kept[0] = 1;
                    exponent += 1;
                }
            }
        }
        kept
    } else {
        raw_digits
    };
    let lead = kept.iter().take_while(|digit| **digit == 0).count();
    if lead == kept.len() {
        return Some(Number::zero());
    }
    Some(Number {
        negative,
        digits: Digits::from_slice(&kept[lead..]),
        exponent,
    })
}

/// Compares two digit vectors by value, ignoring leading zeros.
fn compare_magnitudes(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    let a_start = a.iter().take_while(|d| **d == 0).count();
    let b_start = b.iter().take_while(|d| **d == 0).count();
    let (a, b) = (&a[a_start..], &b[b_start..]);
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Sum of two aligned digit vectors.
#[inline(always)]
fn add_magnitudes(a: &[u8], b: &[u8]) -> Digits {
    let n = a.len().max(b.len());
    let mut out = Digits::zeros(n);
    let mut carry = 0u8;
    for i in 0..n {
        let x = a.len().checked_sub(i + 1).map_or(0, |k| a[k]);
        let y = b.len().checked_sub(i + 1).map_or(0, |k| b[k]);
        let sum = x + y + carry;
        out[n - 1 - i] = sum % 10;
        carry = sum / 10;
    }
    if carry > 0 {
        out.insert_front(carry);
    }
    out
}

/// `a - b`, where `a >= b` by magnitude.
#[inline(always)]
fn sub_magnitudes(a: &[u8], b: &[u8]) -> Digits {
    let n = a.len();
    let mut out = Digits::zeros(n);
    let mut borrow = 0i8;
    for i in 0..n {
        let x = a[n - 1 - i] as i8;
        let y = b.len().checked_sub(i + 1).map_or(0, |k| b[k] as i8);
        let mut d = x - y - borrow;
        if d < 0 {
            d += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[n - 1 - i] = d as u8;
    }
    out
}

/// Removes `count` digits from the low end, by walking the end pointer
/// backwards. Never empties the vector.
fn drop_low_digits(digits: &mut Digits, count: i64) {
    let count = count.max(0) as usize;
    let keep = digits.len().saturating_sub(count).max(1);
    digits.truncate(keep);
}

#[cfg(test)]
mod add_shortcut_tests {
    use super::{SHORTCUT_OFF, exact_small_sum};
    use crate::Number;

    /// The same sum with the shortcut turned off, which is the digit-vector
    /// addition it replaces.
    fn the_long_way(
        left: &Number,
        right: &Number,
        digits: u64,
        subtract: bool,
    ) -> Result<Number, crate::ArithError> {
        SHORTCUT_OFF.set(true);
        let answer = if subtract {
            left.sub(right, digits)
        } else {
            left.add(right, digits)
        };
        SHORTCUT_OFF.set(false);
        answer
    }

    /// Spellings either side of what the shortcut admits: matching and
    /// differing exponents, trailing zeros a Rexx result must keep, a pair
    /// that cancels, and widths around the `i64` bound.
    fn population() -> Vec<Number> {
        [
            "1",
            "-1",
            "1.1",
            "-1.1",
            "1.10",
            "2.2",
            "0.001",
            "9",
            "10",
            "99",
            "100",
            "1E+2",
            "12345",
            "999999999",
            "123456789012345678",
            "0.333333333",
            "0.11111111111111111111",
            "-0.142857142857142857143",
            "99999999999999999999",
            "1.00000000000000000001",
            "5.55555555555555555555E+5",
            "9.99999999E-3",
        ]
        .iter()
        .map(|text| Number::parse(text).expect("a spelling in the population parses"))
        .collect()
    }

    /// `+` and `-` answer the same whether or not the shortcut is allowed to
    /// take them.
    #[test]
    fn the_small_sum_shortcut_answers_what_the_digit_addition_answers() {
        let population = population();
        let mut taken = 0;
        for left in &population {
            for right in &population {
                for digits in [1u64, 2, 3, 5, 9, 10, 18, 20, 21, 37, 40] {
                    for subtract in [false, true] {
                        let shortcut = if subtract {
                            left.sub(right, digits)
                        } else {
                            left.add(right, digits)
                        };
                        let long = the_long_way(left, right, digits, subtract);
                        assert_eq!(
                            shortcut,
                            long,
                            "{} {} {} at DIGITS {digits}",
                            left.format(40),
                            if subtract { "-" } else { "+" },
                            right.format(40)
                        );
                        let working = crate::working_length(digits);
                        let l = left.truncated_to(working);
                        let r = right.truncated_to(working);
                        if !l.is_zero()
                            && !r.is_zero()
                            && exact_small_sum(&l, &r, l.negative, r.negative != subtract, digits)
                                .is_some()
                        {
                            taken += 1;
                        }
                    }
                }
            }
        }
        // A shortcut that declined everything would satisfy the loop above
        // without saying anything.
        assert!(taken > 0, "the shortcut never fired");
    }

    /// Pseudo-random operands of up to forty digits at exponents near each
    /// other, so that most pairs take [`super::aligned_wide_sum`] and its
    /// rounding: both shortcuts on against both off.
    #[test]
    fn the_wide_sum_answers_what_the_digit_addition_answers() {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        let number = |next: &mut dyn FnMut(u64) -> u64| {
            let length = 1 + next(40) as usize;
            let mut text = String::new();
            if next(2) == 0 {
                text.push('-');
            }
            for i in 0..length {
                // Runs of nines and zeros, which is where carries and
                // cancellations live.
                let digit = match next(4) {
                    0 => 9,
                    1 => 0,
                    _ => next(10) as u8,
                };
                text.push(char::from(b'0' + if i == 0 { digit.max(1) } else { digit }));
            }
            // Mostly near each other, sometimes far apart, which is what
            // makes the alignment adjustment shorten an operand.
            let spread = if next(4) == 0 { 90 } else { 30 };
            text.push_str(&format!("E{}", next(spread) as i64 - spread as i64 / 2));
            Number::parse(&text).expect("a generated spelling parses")
        };
        let mut taken = 0;
        for _ in 0..100_000 {
            let left = number(&mut next);
            let right = number(&mut next);
            let digits = 1 + next(40);
            let subtract = next(2) == 0;
            let shortcut = if subtract {
                left.sub(&right, digits)
            } else {
                left.add(&right, digits)
            };
            assert_eq!(
                shortcut,
                the_long_way(&left, &right, digits, subtract),
                "{} {} {} at DIGITS {digits}",
                left.format(40),
                if subtract { "-" } else { "+" },
                right.format(40)
            );
            let working = crate::working_length(digits);
            let (l, r) = (left.truncated_to(working), right.truncated_to(working));
            if super::aligned_wide_sum(&l, &r, l.negative, r.negative != subtract, digits).is_some()
            {
                taken += 1;
            }
        }
        assert!(taken > 20_000, "the wide sum fired {taken} times");
    }

    /// The shortcut declines every operand it must, so that the assertion
    /// above is a statement about a real boundary rather than about an empty
    /// set on one side of it.
    #[test]
    fn the_small_sum_shortcut_declines_what_it_cannot_answer_exactly() {
        let one = Number::parse("1").expect("one");
        let nine = Number::parse("9").expect("nine");
        let wide = Number::parse("123456789012345678").expect("eighteen digits");
        let far = Number::parse("1E+30").expect("a distant exponent");

        // A sum that may carry past the precision has to be rounded, which
        // the shortcut does not do. It is decided on the wider operand plus
        // one rather than on the sum, so `9 + 9` is declined at DIGITS 1
        // although `18` would round to `2E+1` either way.
        assert!(exact_small_sum(&nine, &nine, false, false, 1).is_none());
        assert!(exact_small_sum(&nine, &nine, false, false, 2).is_some());

        // Operands too far apart to share an `i64` mantissa.
        assert!(exact_small_sum(&far, &one, false, false, 40).is_none());

        // A mantissa past what an `i64` holds.
        assert!(exact_small_sum(&wide, &wide, false, false, 40).is_none());

        // A cancelling pair answers the canonical zero rather than one
        // carrying the operands' exponent.
        let tenth = Number::parse("1.1").expect("a tenth");
        assert_eq!(
            exact_small_sum(&tenth, &tenth, false, true, 9).expect("the pair cancels"),
            Number::zero()
        );
    }
}
