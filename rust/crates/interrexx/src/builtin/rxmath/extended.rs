//! The two conversions between degrees or grades and radians, in the x87's
//! extended precision: a 64-bit significand, each operation rounded to
//! nearest-even, the result rounded once more to a double
//! (`extensions-behaviour.md`, "Degrees" and "Arc functions"). Computing them
//! in double arithmetic answers differently: `RxCalcSin(30, 16)` is `0.5` only
//! because ρ rounds to `0.52359877559829892668` in extended precision.

/// A nonzero finite value `significand × 2^exponent`, `significand` with its
/// top bit set, or zero where `significand` is 0.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Extended {
    negative: bool,
    significand: u64,
    exponent: i32,
}

/// The extended value nearest π.
const PI: Extended = Extended {
    negative: false,
    significand: 0xC90F_DAA2_2168_C235,
    exponent: -62,
};

/// `value / 2^shift` rounded to nearest, ties to even, for `shift` from 1
/// up. Answers 0 where everything is shifted out below the half.
fn round_shift(value: u128, shift: u32) -> u128 {
    if shift >= 128 {
        return 0;
    }
    let kept = value >> shift;
    let dropped = value & ((1u128 << shift) - 1);
    let half = 1u128 << (shift - 1);
    if dropped > half || (dropped == half && kept & 1 == 1) {
        kept + 1
    } else {
        kept
    }
}

impl Extended {
    /// `value`, which is finite, exactly.
    fn from_f64(value: f64) -> Extended {
        let negative = value.is_sign_negative();
        let bits = value.to_bits();
        let field = i32::try_from((bits >> 52) & 0x7FF).expect("an 11-bit field");
        let fraction = bits & ((1 << 52) - 1);
        let (mantissa, exponent) = if field == 0 {
            (fraction, -1074)
        } else {
            (fraction | (1 << 52), field - 1075)
        };
        if mantissa == 0 {
            return Extended {
                negative,
                significand: 0,
                exponent: 0,
            };
        }
        let shift = mantissa.leading_zeros();
        Extended {
            negative,
            significand: mantissa << shift,
            exponent: exponent - i32::try_from(shift).expect("a shift below 64"),
        }
    }

    /// A 128-bit `value × 2^exponent` rounded to a 64-bit significand.
    fn rounded(negative: bool, value: u128, exponent: i32) -> Extended {
        if value == 0 {
            return Extended {
                negative,
                significand: 0,
                exponent: 0,
            };
        }
        let width = 128 - value.leading_zeros();
        if width <= 64 {
            let shift = 64 - width;
            return Extended {
                negative,
                significand: u64::try_from(value << shift).expect("64 bits"),
                exponent: exponent - i32::try_from(shift).expect("a shift below 64"),
            };
        }
        let shift = width - 64;
        let mut kept = round_shift(value, shift);
        let mut exponent = exponent + i32::try_from(shift).expect("a shift below 64");
        if kept >> 64 != 0 {
            kept >>= 1;
            exponent += 1;
        }
        Extended {
            negative,
            significand: u64::try_from(kept).expect("64 bits"),
            exponent,
        }
    }

    /// `self × other`, rounded to extended.
    fn times(self, other: Extended) -> Extended {
        let product = u128::from(self.significand) * u128::from(other.significand);
        Extended::rounded(
            self.negative != other.negative,
            product,
            self.exponent + other.exponent,
        )
    }

    /// `self ÷ other`, `other` nonzero, rounded to extended.
    fn divided_by(self, other: Extended) -> Extended {
        let negative = self.negative != other.negative;
        if self.significand == 0 {
            return Extended::rounded(negative, 0, 0);
        }
        // 128 bits of quotient room: the quotient of two 64-bit significands
        // shifted by 64 has 64 or 65 bits, and the remainder decides the
        // rounding of whatever is dropped.
        let dividend = u128::from(self.significand) << 64;
        let divisor = u128::from(other.significand);
        let quotient = dividend / divisor;
        let remainder = dividend % divisor;
        // Two guard bits and a sticky bit carry the remainder into the
        // rounding exactly.
        let sticky = u128::from(remainder != 0);
        let scaled = (quotient << 2) | sticky;
        Extended::rounded(negative, scaled, self.exponent - other.exponent - 66)
    }

    /// The double nearest this value, ties to even, subnormals included.
    fn to_f64(self) -> f64 {
        let sign = if self.negative { -1.0 } else { 1.0 };
        if self.significand == 0 {
            return sign * 0.0;
        }
        // The value is in [2^top, 2^(top+1)).
        let top = self.exponent + 63;
        if top > 1023 {
            return sign * f64::INFINITY;
        }
        let bits = if top >= -1022 {
            let mut mantissa = round_shift(u128::from(self.significand), 11);
            let mut top = top;
            if mantissa >> 53 != 0 {
                mantissa >>= 1;
                top += 1;
            }
            if top > 1023 {
                return sign * f64::INFINITY;
            }
            let field = u64::try_from(top + 1023).expect("a biased exponent");
            (field << 52) | (u64::try_from(mantissa).expect("53 bits") & ((1 << 52) - 1))
        } else {
            // Units of 2^-1074; a carry into bit 52 is the smallest normal,
            // which the same bit pattern spells.
            let shift = u32::try_from(11 - 1022 - top).expect("a positive shift");
            u64::try_from(round_shift(u128::from(self.significand), shift)).expect("53 bits")
        };
        let magnitude = f64::from_bits(bits);
        sign * magnitude
    }
}

/// `r × π ÷ half_circle`, `r` in degrees (`half_circle` 180) or grades (200),
/// each operation in extended precision.
pub(super) fn to_radians(r: f64, half_circle: f64) -> f64 {
    if !r.is_finite() {
        return r * std::f64::consts::PI / half_circle;
    }
    Extended::from_f64(r)
        .times(PI)
        .divided_by(Extended::from_f64(half_circle))
        .to_f64()
}

/// `x ÷ π` in extended precision.
pub(super) fn divide_by_pi(x: f64) -> f64 {
    if !x.is_finite() {
        return x / std::f64::consts::PI;
    }
    Extended::from_f64(x).divided_by(PI).to_f64()
}

#[cfg(test)]
mod tests {
    use super::{Extended, PI, divide_by_pi, to_radians};

    /// The description's witness: 30° in radians rounds, in extended
    /// precision, to `0.52359877559829892668`, whose double has the sine 0.5.
    #[test]
    fn thirty_degrees_has_the_sine_one_half() {
        let rho = to_radians(30.0, 180.0);
        assert_eq!(rho.sin(), 0.5);
        // Double arithmetic does not get there.
        assert_ne!((30.0 * std::f64::consts::PI / 180.0).sin(), 0.5);
    }

    /// π's significand is the 64 bits of π, rounded up at the 65th.
    #[test]
    fn pi_is_the_nearest_extended_value() {
        assert_eq!(PI.to_f64(), std::f64::consts::PI);
        assert_eq!(divide_by_pi(std::f64::consts::PI * 180.0), 180.0);
    }

    /// Products and quotients that are exact stay exact, and a double
    /// survives the round trip through extended.
    #[test]
    fn exact_operations_stay_exact() {
        for value in [1.0, -2.5, 1e-310, 5e-324, 1.7976931348623157e308, 0.1] {
            assert_eq!(Extended::from_f64(value).to_f64(), value);
        }
        let six = Extended::from_f64(2.0).times(Extended::from_f64(3.0));
        assert_eq!(six.to_f64(), 6.0);
        let third = Extended::from_f64(1.0).divided_by(Extended::from_f64(3.0));
        assert_eq!(third.to_f64(), 1.0 / 3.0);
        let quarter = Extended::from_f64(-1.0).divided_by(Extended::from_f64(4.0));
        assert_eq!(quarter.to_f64(), -0.25);
    }
}
