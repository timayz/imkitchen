use time::{Duration, OffsetDateTime, Weekday};

/// Returns the timestamps of the next 4 Mondays from the given timestamp
pub fn next_four_mondays(from_timestamp: i64) -> anyhow::Result<[i64; 4]> {
    let from_date = OffsetDateTime::from_unix_timestamp(from_timestamp)?;
    let mut mondays = [0i64; 4];

    // Get the current weekday
    let current_weekday = from_date.weekday();

    // Calculate days until next Monday
    let days_until_monday = match current_weekday {
        Weekday::Monday => 7, // If today is Monday, get next Monday
        Weekday::Tuesday => 6,
        Weekday::Wednesday => 5,
        Weekday::Thursday => 4,
        Weekday::Friday => 3,
        Weekday::Saturday => 2,
        Weekday::Sunday => 1,
    };

    // Calculate the first Monday
    let first_monday = from_date + Duration::days(days_until_monday);

    // Calculate all 4 Mondays
    for i in 0..4 {
        let monday = first_monday + Duration::weeks(i as i64);
        mondays[i as usize] = monday.unix_timestamp();
    }

    Ok(mondays)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_next_four_mondays_from_monday() {
        // Start from Monday, 2025-01-20
        let monday = datetime!(2025-01-20 12:00:00 UTC);
        let timestamp = monday.unix_timestamp();

        let result = next_four_mondays(timestamp).unwrap();

        // Should get the next 4 Mondays: Jan 27, Feb 3, Feb 10, Feb 17
        let expected = [
            datetime!(2025-01-27 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-03 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-10 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-17 12:00:00 UTC).unix_timestamp(),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_next_four_mondays_from_wednesday() {
        // Start from Wednesday, 2025-01-22
        let wednesday = datetime!(2025-01-22 12:00:00 UTC);
        let timestamp = wednesday.unix_timestamp();

        let result = next_four_mondays(timestamp).unwrap();

        // Should get the next 4 Mondays: Jan 27, Feb 3, Feb 10, Feb 17
        let expected = [
            datetime!(2025-01-27 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-03 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-10 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-17 12:00:00 UTC).unix_timestamp(),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_next_four_mondays_from_sunday() {
        // Start from Sunday, 2025-01-26
        let sunday = datetime!(2025-01-26 12:00:00 UTC);
        let timestamp = sunday.unix_timestamp();

        let result = next_four_mondays(timestamp).unwrap();

        // Should get the next 4 Mondays: Jan 27, Feb 3, Feb 10, Feb 17
        let expected = [
            datetime!(2025-01-27 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-03 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-10 12:00:00 UTC).unix_timestamp(),
            datetime!(2025-02-17 12:00:00 UTC).unix_timestamp(),
        ];

        assert_eq!(result, expected);
    }
}
