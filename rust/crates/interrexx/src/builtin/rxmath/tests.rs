use super::{Trig, Unit, from_radians, strtod, trig};

/// `strtod` stops where a C literal stops: a blank after the sign leaves
/// nothing to read.
#[test]
fn strtod_reads_the_c_literal_prefix() {
    assert_eq!(strtod(b" -4"), -4.0);
    assert_eq!(strtod(b"- 4"), 0.0);
    assert_eq!(strtod(b" + 4 "), 0.0);
    assert_eq!(strtod(b" 9 "), 9.0);
    assert_eq!(strtod(b"4."), 4.0);
    assert_eq!(strtod(b".4e1"), 4.0);
    assert_eq!(strtod(b"1e400"), f64::INFINITY);
    assert_eq!(strtod(b"1e-400"), 0.0);
    assert_eq!(strtod(b"2.0000000001"), 2.000_000_000_1);
}

/// Multiples of a quarter circle answer exactly, with the signs the
/// description's table gives.
#[test]
fn quarter_circles_are_exact() {
    assert_eq!(trig(Trig::Sin, 180.0, Unit::Degrees), 0.0);
    assert_eq!(trig(Trig::Cos, 90.0, Unit::Degrees), 0.0);
    assert_eq!(trig(Trig::Tan, 90.0, Unit::Degrees), f64::INFINITY);
    assert_eq!(trig(Trig::Tan, 270.0, Unit::Degrees), f64::NEG_INFINITY);
    assert_eq!(trig(Trig::Cotan, 180.0, Unit::Degrees), f64::NEG_INFINITY);
    assert_eq!(trig(Trig::Cotan, -180.0, Unit::Degrees), f64::INFINITY);
    assert_eq!(trig(Trig::Sin, 100.0, Unit::Grades), 1.0);
    assert_eq!(trig(Trig::Cos, 200.0, Unit::Grades), -1.0);
    assert_eq!(trig(Trig::Sin, 30.0, Unit::Degrees), 0.5);
    assert_eq!(trig(Trig::Sin, -30.0, Unit::Degrees), -0.5);
    assert_eq!(trig(Trig::Cotan, 0.0, Unit::Radians), f64::INFINITY);
    assert!(trig(Trig::Sin, f64::NAN, Unit::Degrees).is_nan());
    assert!(trig(Trig::Sin, f64::INFINITY, Unit::Degrees).is_nan());
}

/// Arc results in degrees and grades divide by π in extended precision.
#[test]
fn arc_results_convert_exactly() {
    assert_eq!(from_radians(1f64.asin(), Unit::Degrees), 90.0);
    assert_eq!(from_radians(1f64.asin(), Unit::Grades), 100.0);
    assert_eq!(from_radians(0.5f64.asin(), Unit::Degrees), 30.0);
    assert_eq!(from_radians((-1f64).acos(), Unit::Degrees), 180.0);
    assert_eq!(from_radians(f64::INFINITY.atan(), Unit::Degrees), 90.0);
}
