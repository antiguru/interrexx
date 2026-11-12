//! `Scaled`, the loop's machine-integer spelling of a short number, against
//! the `Number` operations it stands for.

use std::cmp::Ordering;

use interrexx_num::{CompareOp, Number, compare_numbers};

/// Spellings chosen for shape: zeros, trailing zeros, both signs, values
/// close together and far apart, and widths either side of small DIGITS.
const VALUES: &[&str] = &[
    "0",
    "1",
    "-1",
    "1.1",
    "-1.1",
    "2.2",
    "1.10",
    "0.5",
    "-0.25",
    "3.0",
    "9.99",
    "0.999",
    "1.00",
    "0.001",
    "100",
    "1E+3",
    "12E+2",
    "123456",
    "1234567",
    "-98765.4321",
    "0.000001",
    "1E-20",
    "5.5E-9",
    "99999999",
    "999999999",
    "123456789012345678",
    "1234567890123456789",
];

fn numbers() -> Vec<Number> {
    VALUES
        .iter()
        .map(|text| Number::parse(text).unwrap())
        .collect()
}

const DIGITS: &[u64] = &[1, 2, 3, 5, 9, 12, 18, 19, 30];

#[test]
fn a_number_round_trips() {
    for value in numbers() {
        match value.scaled() {
            Some(scaled) => assert_eq!(scaled.to_number(), value),
            None => assert!(value.digit_count() > 18),
        }
    }
}

#[test]
fn addition_agrees_with_number_add_wherever_it_answers() {
    let mut answered = 0;
    for left in numbers() {
        for right in numbers() {
            let (Some(a), Some(b)) = (left.scaled(), right.scaled()) else {
                continue;
            };
            for &digits in DIGITS {
                if let Some(sum) = a.add(b, digits) {
                    answered += 1;
                    assert_eq!(
                        Ok(sum.to_number()),
                        left.add(&right, digits),
                        "{left:?} + {right:?} at {digits}"
                    );
                }
            }
        }
    }
    assert!(answered > 500, "only {answered} sums answered");
}

#[test]
fn ordering_agrees_with_the_numeric_comparison_wherever_it_answers() {
    let mut answered = 0;
    for left in numbers() {
        for right in numbers() {
            let (Some(a), Some(b)) = (left.scaled(), right.scaled()) else {
                continue;
            };
            for &digits in DIGITS {
                if let Some(order) = a.order(b, digits) {
                    answered += 1;
                    let less = compare_numbers(&left, &right, digits, 0, CompareOp::Less).unwrap();
                    let equal =
                        compare_numbers(&left, &right, digits, 0, CompareOp::Equal).unwrap();
                    let expected = if less {
                        Ordering::Less
                    } else if equal {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    };
                    assert_eq!(order, expected, "{left:?} vs {right:?} at {digits}");
                }
            }
        }
    }
    assert!(answered > 500, "only {answered} comparisons answered");
}

#[test]
fn the_short_fraction_agrees_with_the_number_s_own() {
    for value in numbers() {
        let Some(scaled) = value.scaled() else {
            continue;
        };
        for &digits in DIGITS {
            for width in [7usize, 12] {
                let mut from_number = vec![0u8; width];
                let mut from_scaled = vec![0u8; width];
                let expected = value.write_short_fraction(digits, &mut from_number);
                let written = scaled.write_short_fraction(digits, &mut from_scaled);
                assert_eq!(written, expected, "{value:?} at {digits}");
                if let Some(length) = written {
                    assert_eq!(from_scaled[..length], from_number[..length]);
                }
            }
        }
    }
}
