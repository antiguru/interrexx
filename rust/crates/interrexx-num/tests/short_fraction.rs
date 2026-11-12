//! `Number::write_short_fraction`, the allocation-free rendering a short
//! fractional result takes, against `Number::format_form`.

use interrexx_num::{Form, Number};

const MANTISSAS: &[&str] = &[
    "1",
    "5",
    "9",
    "10",
    "11",
    "100",
    "105",
    "123",
    "1000",
    "12345",
    "123456",
    "1234567",
    "999999999",
    "1234567890",
];

#[test]
fn agrees_with_the_full_rendering_and_parses_back_unchanged() {
    let mut written = 0;
    for mantissa in MANTISSAS {
        for exponent in -30..=4 {
            for sign in ["", "-"] {
                let spelling = format!("{sign}{mantissa}E{exponent}");
                let value = Number::parse(&spelling).expect("a number");
                for digits in 1..=12u64 {
                    for form in [Form::Scientific, Form::Engineering] {
                        for width in [7usize, 12] {
                            let full = value.format_form(digits, form);
                            let mut out = vec![0u8; width];
                            match value.write_short_fraction(digits, &mut out) {
                                Some(length) => {
                                    written += 1;
                                    let short = &out[..length];
                                    assert_eq!(short, full.as_bytes(), "{spelling} at {digits}");
                                    let back = Number::parse_bytes(short).expect("numeric");
                                    assert_eq!(back, value, "{spelling} does not parse back");
                                    assert_eq!(back.digit_count(), value.digit_count());
                                }
                                None => {
                                    let declined_rightly = !full.contains('.')
                                        || full.contains('E')
                                        || value.digit_count() as u64 > digits
                                        || full.len() > width;
                                    assert!(
                                        declined_rightly,
                                        "{spelling} at {digits} declined, renders {full}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(written > 1000, "only {written} cases took the short rendering");
}
