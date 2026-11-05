//! The `rxmath` extension library, run in Rust: `MathLoadFuncs`,
//! `MathDropFuncs` and the sixteen `RxCalc` routines, as
//! `docs/design/extensions.md` describes them.
//!
//! Every result is a double rendered by [`crate::dispatch::library::double_text`] at the
//! routine's precision. The trigonometric routines reduce a degree or grade
//! angle before any library call, and form the radian angle in the x87's
//! extended precision, which [`extended`] carries out in integers.

mod extended;

use interrexx_core::{Decoded, ObjRef};
use interrexx_num::Number;

use crate::error::Raised;
use crate::{Failure, Interp};

/// The most significant digits a result carries.
const MAX_PRECISION: u64 = 16;

/// 88.922 where more arguments were passed than `arity`. An omitted argument
/// at the end does not count: measured, oracle, `RxCalcSqrt(4,,,,)` is `2`.
pub(crate) fn check_arity(args: &[Option<ObjRef>], arity: usize) -> Result<(), Failure> {
    let passed = args
        .iter()
        .rposition(Option::is_some)
        .map_or(0, |at| at + 1);
    if passed > arity {
        return Err(Raised::too_many_external_arguments(arity).into());
    }
    Ok(())
}

/// The argument at `position`, 88.901 where it is omitted.
pub(crate) fn required(args: &[Option<ObjRef>], position: usize) -> Result<ObjRef, Failure> {
    args.get(position - 1)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_native_argument(position).into())
}

/// A floating argument: a Rexx number, read as C's `strtod` reads its text,
/// or exactly `nan`, `+infinity` or `-infinity`. 88.921 for anything else.
pub(crate) fn double_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<f64, Failure> {
    let value = required(args, position)?;
    if let Decoded::SmallInt(number) = value.decode() {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a whole number converts to the nearest double, as C's does"
        )]
        return Ok(number as f64);
    }
    let text = interp.required_string_or_nil(value)?;
    let bytes = interp.to_text(text).into_owned();
    if Number::parse_bytes(&bytes).is_some() {
        return Ok(strtod(&bytes));
    }
    match &bytes[..] {
        b"nan" => Ok(f64::NAN),
        b"+infinity" => Ok(f64::INFINITY),
        b"-infinity" => Ok(f64::NEG_INFINITY),
        _ => {
            let found = interp.native_found(value);
            Err(Raised::native_argument_not_a_double(position, &found).into())
        }
    }
}

/// C's `strtod` over a valid Rexx number: leading white space skipped, then
/// the longest prefix that is a decimal floating literal, and 0 where there is
/// none. Measured, oracle: `RxCalcPower(' -4', 1)` is `-4` and
/// `RxCalcPower('- 4', 1)` is `0`, because a blank after the sign ends the
/// literal before it starts.
fn strtod(text: &[u8]) -> f64 {
    let start = text
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(text.len());
    let text = &text[start..];
    let mut end = 0;
    if matches!(text.first(), Some(b'+' | b'-')) {
        end = 1;
    }
    let digits = |from: usize| {
        text[from..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count()
    };
    let whole = digits(end);
    end += whole;
    let mut fraction = 0;
    if text.get(end) == Some(&b'.') {
        fraction = digits(end + 1);
        if whole + fraction > 0 {
            end += 1 + fraction;
        }
    }
    if whole + fraction == 0 {
        return 0.0;
    }
    if matches!(text.get(end), Some(b'e' | b'E')) {
        let mut at = end + 1;
        if matches!(text.get(at), Some(b'+' | b'-')) {
            at += 1;
        }
        let exponent = digits(at);
        if exponent > 0 {
            end = at + exponent;
        }
    }
    let literal = std::str::from_utf8(&text[..end]).expect("an ASCII literal");
    literal
        .parse()
        .unwrap_or_else(|_| unreachable!("a decimal literal parses: {literal}"))
}

/// The precision at `position`: a positive whole number, at most
/// [`MAX_PRECISION`], or where omitted the caller's `NUMERIC DIGITS` capped
/// the same way. 88.905 for anything that is not a positive whole number.
fn precision(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<u64, Failure> {
    let digits = match args.get(position - 1).copied().flatten() {
        Some(value) => match super::rexxutil::stems::whole_value(interp, value, 1)? {
            Some(given) => u64::try_from(given).unwrap_or(MAX_PRECISION),
            None => {
                let found = interp.native_found(value);
                return Err(Raised::native_argument_not_positive(position, &found).into());
            }
        },
        None => interp.activation().settings.digits(),
    };
    Ok(digits.min(MAX_PRECISION))
}

/// The unit an angle argument or an arc result is in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Unit {
    Degrees,
    Radians,
    Grades,
}

impl Unit {
    /// The angle of a full circle, or `None` for radians, which are never
    /// reduced.
    fn circle(self) -> Option<f64> {
        match self {
            Unit::Degrees => Some(360.0),
            Unit::Grades => Some(400.0),
            Unit::Radians => None,
        }
    }
}

/// The units argument at `position`: its first character, caselessly, `D`,
/// `R` or `G`, degrees where omitted. 88.916 for anything else, the null
/// string included.
fn unit(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Result<Unit, Failure> {
    let Some(value) = args.get(position - 1).copied().flatten() else {
        return Ok(Unit::Degrees);
    };
    let Some(converted) = interp.blamed_string_conversion(value)? else {
        return Err(Raised::native_argument_needs_a_string_value(position).into());
    };
    let text = interp.to_text(converted).into_owned();
    match text.first().map(u8::to_ascii_uppercase) {
        Some(b'D') => Ok(Unit::Degrees),
        Some(b'R') => Ok(Unit::Radians),
        Some(b'G') => Ok(Unit::Grades),
        _ => {
            let mut raised = Raised::not_one_of(position, b"D, R, or G", &text);
            raised.delivery.lineless = true;
            Err(raised.into())
        }
    }
}

/// `value` as the routine's answer, at `precision` significant digits.
fn answer(interp: &mut Interp, value: f64, precision: u64) -> ObjRef {
    let precision = usize::try_from(precision).unwrap_or(usize::MAX);
    interp.text(&crate::dispatch::library::double_text(value, precision))
}

/// `MathLoadFuncs([version])`: with any argument, the library's banner on
/// standard output. It registers nothing; loading the library did that.
pub(crate) fn load_funcs(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    if args.first().copied().flatten().is_some() {
        interp.write_out(
            concat!(
                "rxmath (interrexx ",
                env!("CARGO_PKG_VERSION"),
                ") - REXX mathematical function library\n\n"
            )
            .as_bytes(),
        );
    }
    Ok(interp.text(b""))
}

/// `MathDropFuncs()`: nothing.
pub(crate) fn drop_funcs(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    Ok(interp.text(b""))
}

/// `RxCalcPi([precision])`: the double nearest π.
pub(crate) fn pi(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let precision = precision(interp, args, 1)?;
    Ok(answer(interp, std::f64::consts::PI, precision))
}

/// One of the routines taking `x [, precision]` and answering `f(x)`.
fn unary(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    function: fn(f64) -> f64,
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let x = double_argument(interp, args, 1)?;
    let precision = precision(interp, args, 2)?;
    Ok(answer(interp, function(x), precision))
}

/// `RxCalcSqrt(x [, precision])`.
pub(crate) fn sqrt(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    unary(interp, args, f64::sqrt)
}

/// `RxCalcExp(x [, precision])`.
pub(crate) fn exp(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    unary(interp, args, f64::exp)
}

/// `RxCalcLog(x [, precision])`, the natural logarithm.
pub(crate) fn log(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    unary(interp, args, f64::ln)
}

/// `RxCalcLog10(x [, precision])`.
pub(crate) fn log10(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    unary(interp, args, f64::log10)
}

/// `RxCalcSinH(x [, precision])`.
pub(crate) fn sinh(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    unary(interp, args, f64::sinh)
}

/// `RxCalcCosH(x [, precision])`.
pub(crate) fn cosh(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    unary(interp, args, f64::cosh)
}

/// `RxCalcTanH(x [, precision])`.
pub(crate) fn tanh(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    unary(interp, args, f64::tanh)
}

/// `RxCalcPower(x, y [, precision])`: C's `pow`.
pub(crate) fn power(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let x = double_argument(interp, args, 1)?;
    let y = double_argument(interp, args, 2)?;
    let precision = precision(interp, args, 3)?;
    Ok(answer(interp, x.powf(y), precision))
}

/// The four trigonometric functions of an angle.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Trig {
    Sin,
    Cos,
    Tan,
    Cotan,
}

impl Trig {
    /// The function of `rho`, in radians, with no reduction.
    fn of(self, rho: f64) -> f64 {
        match self {
            Trig::Sin => rho.sin(),
            Trig::Cos => rho.cos(),
            Trig::Tan => rho.tan(),
            Trig::Cotan => 1.0 / rho.tan(),
        }
    }
}

/// `function` of `angle` in `unit`.
///
/// A degree or grade angle is reduced first, so that a multiple of a quarter
/// circle answers exactly: its magnitude modulo the circle picks a quadrant,
/// the angle's offset from that quadrant's axis is converted to radians in
/// extended precision, and the function comes from the sine, cosine or
/// tangent of that offset with the quadrant's sign
/// (`extensions-behaviour.md`, "Degrees" and "Grades").
pub(crate) fn trig(function: Trig, angle: f64, unit: Unit) -> f64 {
    let Some(circle) = unit.circle() else {
        return function.of(angle);
    };
    let s = if angle < 0.0 { -1.0 } else { 1.0 };
    let quarter = circle / 4.0;
    let eighth = circle / 8.0;
    let a = angle.abs() % circle;
    let radians = |r: f64| extended::to_radians(r, circle / 2.0);
    if a <= eighth {
        let rho = radians(a);
        match function {
            Trig::Sin => s * rho.sin(),
            Trig::Cos => rho.cos(),
            Trig::Tan => s * rho.tan(),
            Trig::Cotan => s / rho.tan(),
        }
    } else if a < quarter + eighth {
        let rho = radians(quarter - a);
        match function {
            Trig::Sin => s * rho.cos(),
            Trig::Cos => rho.sin(),
            Trig::Tan => s / rho.tan(),
            Trig::Cotan => s * rho.tan(),
        }
    } else if a <= 2.0 * quarter + eighth {
        let rho = radians(2.0 * quarter - a);
        match function {
            Trig::Sin => s * rho.sin(),
            Trig::Cos => -rho.cos(),
            Trig::Tan => -s * rho.tan(),
            Trig::Cotan => -s / rho.tan(),
        }
    } else if a < 3.0 * quarter + eighth {
        let rho = radians(a - 3.0 * quarter);
        match function {
            Trig::Sin => -s * rho.cos(),
            Trig::Cos => rho.sin(),
            Trig::Tan => -s / rho.tan(),
            Trig::Cotan => -s * rho.tan(),
        }
    } else {
        let rho = radians(a - circle);
        match function {
            Trig::Sin => s * rho.sin(),
            Trig::Cos => rho.cos(),
            Trig::Tan => s * rho.tan(),
            Trig::Cotan => s / rho.tan(),
        }
    }
}

/// One of the routines taking `angle [, precision [, units]]`.
fn trig_routine(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    function: Trig,
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let angle = double_argument(interp, args, 1)?;
    let precision = precision(interp, args, 2)?;
    let unit = unit(interp, args, 3)?;
    Ok(answer(interp, trig(function, angle, unit), precision))
}

/// `RxCalcSin(angle [, precision [, units]])`.
pub(crate) fn sin(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    trig_routine(interp, args, Trig::Sin)
}

/// `RxCalcCos(angle [, precision [, units]])`.
pub(crate) fn cos(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    trig_routine(interp, args, Trig::Cos)
}

/// `RxCalcTan(angle [, precision [, units]])`.
pub(crate) fn tan(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    trig_routine(interp, args, Trig::Tan)
}

/// `RxCalcCotan(angle [, precision [, units]])`.
pub(crate) fn cotan(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    trig_routine(interp, args, Trig::Cotan)
}

/// `v`, an angle in radians, in `unit`: for degrees `v × 180` in double
/// arithmetic, then divided by π in extended precision; for grades the same
/// with 200.
pub(crate) fn from_radians(v: f64, unit: Unit) -> f64 {
    match unit.circle() {
        None => v,
        Some(circle) => extended::divide_by_pi(v * (circle / 2.0)),
    }
}

/// One of the routines taking `x [, precision [, units]]` and answering an
/// angle.
fn arc_routine(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    function: fn(f64) -> f64,
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let x = double_argument(interp, args, 1)?;
    let precision = precision(interp, args, 2)?;
    let unit = unit(interp, args, 3)?;
    Ok(answer(interp, from_radians(function(x), unit), precision))
}

/// `RxCalcArcSin(x [, precision [, units]])`.
pub(crate) fn arc_sin(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    arc_routine(interp, args, f64::asin)
}

/// `RxCalcArcCos(x [, precision [, units]])`.
pub(crate) fn arc_cos(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    arc_routine(interp, args, f64::acos)
}

/// `RxCalcArcTan(x [, precision [, units]])`.
pub(crate) fn arc_tan(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    arc_routine(interp, args, f64::atan)
}

#[cfg(test)]
mod tests;
