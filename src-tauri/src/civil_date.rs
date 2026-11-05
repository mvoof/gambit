//! Unix seconds to the PGN date. One conversion is not worth a date crate.

const SECONDS_PER_DAY: u64 = 86_400;
const DAYS_PER_ERA: u64 = 146_097;
const YEARS_PER_ERA: u64 = 400;
/// Days from 0000-03-01 to 1970-01-01.
const EPOCH_SHIFT_DAYS: u64 = 719_468;

/// `YYYY.MM.DD`, UTC. Howard Hinnant's `civil_from_days`, for dates after 1970.
pub fn pgn_date(unix_seconds: u64) -> String {
    let days = unix_seconds / SECONDS_PER_DAY + EPOCH_SHIFT_DAYS;
    let era = days / DAYS_PER_ERA;
    let day_of_era = days - era * DAYS_PER_ERA;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * YEARS_PER_ERA + u64::from(month <= 2);

    format!("{year:04}.{month:02}.{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_dates() {
        assert_eq!(pgn_date(0), "1970.01.01");
        assert_eq!(pgn_date(951_782_400), "2000.02.29");
        assert_eq!(pgn_date(1_709_164_800), "2024.02.29");
        assert_eq!(pgn_date(1_790_000_000), "2026.09.21");
    }
}
