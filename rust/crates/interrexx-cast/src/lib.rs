//! Numeric conversions that say what they keep.
//!
//! The workspace denies `as` between numeric types (`clippy::as_conversions`):
//! an `as` reads the same whether it is exact, rounds or drops bits, and which
//! one it is can change with the target. This crate is the one place such a
//! conversion is written, each under a name that says which it is:
//!
//! * [`CastFrom`] / [`CastInto`]: exact, for the pairs `From` does not cover
//!   because they involve `usize` or `isize`. A pair is implemented only on the
//!   targets where it is exact: `u64` to `usize` exists on a 64-bit target and
//!   not on a 32-bit one, so code that needs it there does not compile and has
//!   to say what it does with a value that does not fit (`try_from`).
//! * [`CastLossy`]: to a float, rounding to the nearest representable value.
//! * [`CastTruncate`]: to a narrower integer of the same signedness, keeping
//!   the low bits, for hashing, masking and fixed-width fields.
//! * [`TryCastFrom`]: from a float to an integer, toward zero, or `None` where
//!   the value is not finite or does not fit.
//! * `const fn`s for the exact widenings a constant needs, where neither a
//!   trait nor `From` can be called.
//!
//! A change of signedness at the same width is `cast_signed` and
//! `cast_unsigned` in `std`. A pointer's address is `addr`, and an address made
//! into a pointer is `std::ptr::without_provenance`.

// The conversions are written here, once each, behind the names above.
#![allow(clippy::as_conversions)]

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("interrexx builds for 32-bit and 64-bit targets only");

/// An exact conversion that `From` does not provide.
pub trait CastFrom<T> {
    /// `from`, unchanged in value.
    fn cast_from(from: T) -> Self;
}

/// [`CastFrom`] from the other side, as `Into` is to `From`.
pub trait CastInto<T> {
    /// `self`, unchanged in value.
    fn cast_into(self) -> T;
}

impl<T, U: CastFrom<T>> CastInto<U> for T {
    #[inline(always)]
    fn cast_into(self) -> U {
        U::cast_from(self)
    }
}

/// A conversion to a float, rounding to the nearest value it holds.
pub trait CastLossy<T> {
    /// `from`, rounded to the nearest value `Self` holds.
    fn cast_lossy(from: T) -> Self;
}

/// A conversion to a narrower integer of the same signedness that keeps the
/// low bits: two's complement wrapping, as for a hash or a fixed-width field.
pub trait CastTruncate<T> {
    /// The low bits of `from` that `Self` holds.
    fn cast_truncate(from: T) -> Self;
}

/// A conversion from a float to an integer, toward zero.
pub trait TryCastFrom<T>: Sized {
    /// `from` with its fraction dropped, or `None` where it is not finite or
    /// its whole part does not fit `Self`.
    fn try_cast_from(from: T) -> Option<Self>;
}

macro_rules! impls {
    ($trait:ident, $method:ident: $($from:ty => $($to:ty),+;)*) => {
        $($(
            impl $trait<$from> for $to {
                #[inline(always)]
                fn $method(from: $from) -> $to {
                    from as $to
                }
            }
        )+)*
    };
}

// Exact on every supported target: `usize` and `isize` are 32 or 64 bits.
impls! {
    CastFrom, cast_from:
    u8 => usize, isize;
    u16 => usize, isize;
    u32 => usize;
    i8 => isize;
    i16 => isize;
    i32 => isize;
    usize => u64, u128, i128;
    isize => i64, i128;
}

// Exact only where `usize` and `isize` are 64 bits.
#[cfg(target_pointer_width = "64")]
impls! {
    CastFrom, cast_from:
    u32 => isize;
    u64 => usize;
    i64 => isize;
}

// Exact only where `usize` and `isize` are 32 bits.
#[cfg(target_pointer_width = "32")]
impls! {
    CastFrom, cast_from:
    usize => u32, i64;
    isize => i32;
}

impls! {
    CastLossy, cast_lossy:
    u32 => f32;
    i32 => f32;
    u64 => f32, f64;
    i64 => f32, f64;
    usize => f32, f64;
    isize => f32, f64;
    u128 => f32, f64;
    i128 => f32, f64;
    f64 => f32;
}

impls! {
    CastTruncate, cast_truncate:
    u16 => u8;
    u32 => u8, u16;
    u64 => u8, u16, u32, usize;
    u128 => u8, u16, u32, u64, usize;
    usize => u8, u16, u32;
    i16 => i8;
    i32 => i8, i16;
    i64 => i8, i16, i32, isize;
    i128 => i8, i16, i32, i64, isize;
    isize => i8, i16, i32;
}

macro_rules! try_from_float {
    ($($float:ty => $($to:ty),+;)*) => {
        $($(
            impl TryCastFrom<$float> for $to {
                #[inline]
                fn try_cast_from(from: $float) -> Option<$to> {
                    // Both bounds are powers of two, which a float holds
                    // exactly: the least value, and one past the greatest.
                    let low = if <$to>::MIN == 0 { 0.0 } else { -(2.0 as $float).powi(<$to>::BITS as i32 - 1) };
                    let high = if <$to>::MIN == 0 {
                        (2.0 as $float).powi(<$to>::BITS as i32)
                    } else {
                        (2.0 as $float).powi(<$to>::BITS as i32 - 1)
                    };
                    let whole = from.trunc();
                    (whole >= low && whole < high).then(|| whole as $to)
                }
            }
        )+)*
    };
}

try_from_float! {
    f64 => u8, u16, u32, u64, usize, i8, i16, i32, i64, isize;
    f32 => u8, u16, u32, u64, usize, i8, i16, i32, i64, isize;
}

macro_rules! const_widen {
    ($($name:ident: $from:ty => $to:ty;)*) => {
        $(
            #[doc = concat!("`", stringify!($from), "` as `", stringify!($to), "`, exactly, for a constant.")]
            #[inline(always)]
            #[must_use]
            pub const fn $name(from: $from) -> $to {
                from as $to
            }
        )*
    };
}

const_widen! {
    u16_from_u8: u8 => u16;
    u32_from_u8: u8 => u32;
    u32_from_u16: u16 => u32;
    u32_from_char: char => u32;
    u64_from_u8: u8 => u64;
    u64_from_u16: u16 => u64;
    u64_from_u32: u32 => u64;
    u64_from_usize: usize => u64;
    u128_from_u64: u64 => u128;
    usize_from_u8: u8 => usize;
    usize_from_u16: u16 => usize;
    usize_from_u32: u32 => usize;
    i32_from_i8: i8 => i32;
    i32_from_i16: i16 => i32;
    i32_from_u16: u16 => i32;
    i64_from_i32: i32 => i64;
    i64_from_u32: u32 => i64;
    i64_from_isize: isize => i64;
    i128_from_i64: i64 => i128;
    i128_from_u64: u64 => i128;
    f64_from_u32: u32 => f64;
    f64_from_i32: i32 => f64;
}

#[cfg(target_pointer_width = "64")]
const_widen! {
    usize_from_u64: u64 => usize;
    isize_from_i64: i64 => isize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_conversions_keep_the_value() {
        assert_eq!(usize::cast_from(u32::MAX), 4_294_967_295);
        assert_eq!(u64::cast_from(usize::MAX), usize::MAX as u64);
        assert_eq!(i64::cast_from(isize::MIN), isize::MIN as i64);
        let into: usize = 7u16.cast_into();
        assert_eq!(into, 7);
        #[cfg(target_pointer_width = "64")]
        assert_eq!(usize::cast_from(u64::MAX), usize::MAX);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(u32::cast_from(usize::MAX), u32::MAX);
    }

    #[test]
    fn truncation_keeps_the_low_bits() {
        assert_eq!(u32::cast_truncate(0x1234_5678_9abc_def0_u64), 0x9abc_def0);
        assert_eq!(u8::cast_truncate(0x1ff_u16), 0xff);
        assert_eq!(i8::cast_truncate(-129_i32), 127);
        assert_eq!(u64::cast_truncate(u128::MAX), u64::MAX);
    }

    #[test]
    fn a_lossy_conversion_rounds_to_nearest() {
        assert_eq!(f64::cast_lossy(u64::MAX), 18_446_744_073_709_551_616.0);
        assert_eq!(f64::cast_lossy((1u64 << 53) + 1), 9_007_199_254_740_992.0);
        assert_eq!(f32::cast_lossy(0.1f64), 0.1f32);
    }

    #[test]
    fn a_float_converts_toward_zero_within_the_range() {
        assert_eq!(i64::try_cast_from(-2.9f64), Some(-2));
        assert_eq!(u8::try_cast_from(255.9f64), Some(255));
        assert_eq!(u8::try_cast_from(256.0f64), None);
        assert_eq!(u8::try_cast_from(-0.5f64), Some(0));
        assert_eq!(u8::try_cast_from(-1.0f64), None);
        assert_eq!(i8::try_cast_from(-128.0f64), Some(-128));
        assert_eq!(i8::try_cast_from(-129.0f64), None);
        assert_eq!(u64::try_cast_from(18_446_744_073_709_551_616.0f64), None);
        assert_eq!(i64::try_cast_from(f64::NAN), None);
        assert_eq!(u32::try_cast_from(f64::INFINITY), None);
        assert_eq!(u32::try_cast_from(4_294_967_295.0f64), Some(u32::MAX));
    }

    #[test]
    fn the_const_widenings_are_exact() {
        const WIDE: u64 = u64_from_u32(u32::MAX);
        assert_eq!(WIDE, 4_294_967_295);
        assert_eq!(i64_from_i32(i32::MIN), -2_147_483_648);
        assert_eq!(u32_from_char('\u{10ffff}'), 0x10_ffff);
    }
}
