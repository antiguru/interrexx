//! Whole-number kernels over base-10^9 limbs, for operands long enough that
//! working one decimal digit at a time dominates.
//!
//! A limb vector is least significant first, each limb below [`BASE`]. Only
//! exact integer results leave this module: the decimal digit vectors go in
//! and come out unchanged in meaning, so rounding, exponents and the stopping
//! rules stay with the digit-level callers.

/// Nine decimal digits per limb: the product of two limbs plus two more
/// limbs stays inside a `u64`.
const BASE: u64 = 1_000_000_000;
const LIMB_DIGITS: usize = 9;

/// Below this many limbs in the shorter operand, Karatsuba's extra additions
/// cost more than the multiplications it saves.
const KARATSUBA_LIMBS: usize = 48;

/// The limbs of the whole number spelt by `digits` (most significant first)
/// followed by `zeros` zero digits.
pub(crate) fn from_digits(digits: &[u8], zeros: usize) -> Vec<u32> {
    let total = digits.len() + zeros;
    let mut limbs = Vec::with_capacity(total.div_ceil(LIMB_DIGITS));
    // Whole zero limbs first, then the remainder of the padding shifts the
    // digits within their limbs.
    limbs.resize(zeros / LIMB_DIGITS, 0);
    let partial = zeros % LIMB_DIGITS;
    let mut limb: u32 = 0;
    let mut scale: u32 = 10u32.pow(partial as u32);
    let mut filled = partial;
    for &digit in digits.iter().rev() {
        limb += u32::from(digit) * scale;
        filled += 1;
        if filled == LIMB_DIGITS {
            limbs.push(limb);
            limb = 0;
            scale = 1;
            filled = 0;
        } else {
            scale *= 10;
        }
    }
    if filled > 0 {
        limbs.push(limb);
    }
    trim(&mut limbs);
    limbs
}

/// The decimal digits of `limbs`, most significant first, with no leading
/// zeros (a zero is the single digit `0`).
pub(crate) fn to_digits(limbs: &[u32]) -> Vec<u8> {
    let mut top = limbs.len();
    while top > 1 && limbs[top - 1] == 0 {
        top -= 1;
    }
    if top == 0 {
        return vec![0];
    }
    let mut out = Vec::with_capacity(top * LIMB_DIGITS);
    let mut buf = [0u8; LIMB_DIGITS];
    let mut lead = limbs[top - 1];
    let mut len = 0;
    loop {
        buf[len] = (lead % 10) as u8;
        lead /= 10;
        len += 1;
        if lead == 0 {
            break;
        }
    }
    out.extend(buf[..len].iter().rev());
    for &limb in limbs[..top - 1].iter().rev() {
        let mut v = limb;
        for slot in buf.iter_mut().rev() {
            *slot = (v % 10) as u8;
            v /= 10;
        }
        out.extend_from_slice(&buf);
    }
    out
}

fn trim(limbs: &mut Vec<u32>) {
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
}

/// `a * b`, exact, with `a.len() + b.len()` limbs (possibly a leading zero).
pub(crate) fn mul(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = vec![0u32; a.len() + b.len()];
    mul_into(&mut out, a, b);
    out
}

/// `out = a * b`, where `out` is zeroed and `a.len() + b.len()` limbs long.
fn mul_into(out: &mut [u32], a: &[u32], b: &[u32]) {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if short.len() < KARATSUBA_LIMBS {
        schoolbook(out, short, long);
    } else if long.len() >= 2 * short.len() {
        // Lopsided: Karatsuba on short-sized slices of the longer operand.
        let mut scratch = vec![0u32; 2 * short.len()];
        for (index, chunk) in long.chunks(short.len()).enumerate() {
            let piece = &mut scratch[..short.len() + chunk.len()];
            piece.fill(0);
            mul_into(piece, short, chunk);
            add_into(&mut out[index * short.len()..], piece);
        }
    } else {
        karatsuba(out, a, b);
    }
}

fn schoolbook(out: &mut [u32], a: &[u32], b: &[u32]) {
    for (i, &x) in a.iter().enumerate() {
        if x == 0 {
            continue;
        }
        let x = u64::from(x);
        let mut carry = 0u64;
        for (j, &y) in b.iter().enumerate() {
            let t = u64::from(out[i + j]) + x * u64::from(y) + carry;
            out[i + j] = (t % BASE) as u32;
            carry = t / BASE;
        }
        let mut k = i + b.len();
        while carry != 0 {
            let t = u64::from(out[k]) + carry;
            out[k] = (t % BASE) as u32;
            carry = t / BASE;
            k += 1;
        }
    }
}

/// Karatsuba on two operands of comparable length, split at half the
/// shorter one.
fn karatsuba(out: &mut [u32], a: &[u32], b: &[u32]) {
    let half = a.len().min(b.len()) / 2;
    let (a0, a1) = a.split_at(half);
    let (b0, b1) = b.split_at(half);

    // z0 = a0*b0 and z2 = a1*b1 land in their own places of `out`.
    let (low, high) = out.split_at_mut(2 * half);
    mul_into(low, a0, b0);
    mul_into(&mut high[..a1.len() + b1.len()], a1, b1);

    // z1 = (a0+a1)(b0+b1) - z0 - z2, added in at `half`.
    let sa = sum(a0, a1);
    let sb = sum(b0, b1);
    let mut z1 = vec![0u32; sa.len() + sb.len()];
    mul_into(&mut z1, &sa, &sb);
    sub_from(&mut z1, &out[..2 * half]);
    sub_from(&mut z1, &out[2 * half..2 * half + a1.len() + b1.len()]);
    add_into(&mut out[half..], &z1);
}

fn sum(a: &[u32], b: &[u32]) -> Vec<u32> {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let mut out = Vec::with_capacity(long.len() + 1);
    let mut carry = 0u32;
    for (i, &l) in long.iter().enumerate() {
        let mut t = l + short.get(i).copied().unwrap_or(0) + carry;
        carry = u32::from(t >= BASE as u32);
        if carry != 0 {
            t -= BASE as u32;
        }
        out.push(t);
    }
    if carry != 0 {
        out.push(carry);
    }
    out
}

/// `acc += addend`, the carry running on into `acc`'s higher limbs. The
/// caller guarantees the sum fits.
fn add_into(acc: &mut [u32], addend: &[u32]) {
    let mut carry = 0u32;
    let mut i = 0;
    while i < addend.len() || carry != 0 {
        let mut t = acc[i] + addend.get(i).copied().unwrap_or(0) + carry;
        carry = u32::from(t >= BASE as u32);
        if carry != 0 {
            t -= BASE as u32;
        }
        acc[i] = t;
        i += 1;
    }
}

/// `acc -= sub`, which the caller guarantees does not go negative; any limbs
/// of `sub` past `acc`'s length are zero.
fn sub_from(acc: &mut [u32], sub: &[u32]) {
    let mut borrow = 0u32;
    let mut i = 0;
    while i < sub.len() || borrow != 0 {
        let s = sub.get(i).copied().unwrap_or(0) + borrow;
        if i >= acc.len() {
            debug_assert_eq!(s, 0, "the difference is not negative");
            i += 1;
            continue;
        }
        if acc[i] >= s {
            acc[i] -= s;
            borrow = 0;
        } else {
            acc[i] = acc[i] + BASE as u32 - s;
            borrow = 1;
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// xorshift64*, so a failure reproduces from the seed.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
        /// A digit string with a non-zero lead, biased toward runs of 9s and
        /// 0s, which is where carries and borrows go wrong.
        fn digits(&mut self, len: usize) -> Vec<u8> {
            let style = self.below(4);
            (0..len)
                .map(|i| {
                    let d = match style {
                        0 => 9,
                        1 if i > 0 => 0,
                        2 if self.below(3) == 0 => 9,
                        _ => self.below(10) as u8,
                    };
                    if i == 0 && d == 0 { 1 } else { d }
                })
                .collect()
        }
    }

    /// Digit-by-digit reference product.
    fn reference_mul(a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut out = vec![0u32; a.len() + b.len()];
        for (i, x) in a.iter().rev().enumerate() {
            for (j, y) in b.iter().rev().enumerate() {
                out[i + j] += u32::from(*x) * u32::from(*y);
            }
        }
        let mut carry = 0;
        for slot in out.iter_mut() {
            let v = *slot + carry;
            *slot = v % 10;
            carry = v / 10;
        }
        let mut digits: Vec<u8> = out.iter().rev().map(|d| *d as u8).collect();
        let lead = digits.iter().take_while(|d| **d == 0).count();
        digits.drain(..lead.min(digits.len() - 1));
        digits
    }

    #[test]
    fn digits_round_trip_through_limbs() {
        let mut rng = Rng(7);
        for len in 1..60 {
            for zeros in [0, 1, 8, 9, 10, 17, 18, 19] {
                let digits = rng.digits(len);
                let limbs = from_digits(&digits, zeros);
                let mut expected = digits.clone();
                expected.extend(std::iter::repeat_n(0, zeros));
                assert_eq!(to_digits(&limbs), expected, "{digits:?} + {zeros} zeros");
            }
        }
        assert_eq!(to_digits(&from_digits(&[0], 0)), vec![0]);
        assert_eq!(to_digits(&[]), vec![0]);
    }

    #[test]
    fn the_limb_product_is_the_digit_product() {
        let mut rng = Rng(11);
        // Lengths either side of the Karatsuba threshold, and lopsided pairs.
        let karatsuba_digits = KARATSUBA_LIMBS * LIMB_DIGITS;
        let lengths = [
            1,
            9,
            10,
            50,
            karatsuba_digits - 1,
            karatsuba_digits,
            karatsuba_digits + 9,
            2 * karatsuba_digits + 5,
            5 * karatsuba_digits + 3,
        ];
        for &la in &lengths {
            for &lb in &lengths {
                for _ in 0..3 {
                    let a = rng.digits(la);
                    let b = rng.digits(lb);
                    let product = mul(&from_digits(&a, 0), &from_digits(&b, 0));
                    assert_eq!(to_digits(&product), reference_mul(&a, &b), "{la} x {lb}");
                }
            }
        }
    }
}
