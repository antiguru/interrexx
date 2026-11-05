//! The build date as `d Mon yyyy`, shared by `build.rs` and the unit test in `src/version.rs`.

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// The UTC date of `seconds` since the Unix epoch, the day without a leading zero.
pub fn format(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    // Civil-from-days over eras of 400 years starting 0000-03-01.
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    let month = usize::try_from(month - 1).expect("a month in 1..=12");
    format!("{day} {} {year}", MONTHS[month])
}
