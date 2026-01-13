use time::{Duration, OffsetDateTime, Weekday};
use time_tz::{ToTimezone, timezones};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Week {
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

pub fn now(tz: impl Into<String>) -> OffsetDateTime {
    let tz = tz.into();
    // let mut now = OffsetDateTime::now_utc();
    let mut now = time::macros::datetime!(2026-02-03 00:00:00 UTC);

    if let Some(tz) = timezones::get_by_name(&tz) {
        now = now.to_timezone(tz);
    }

    now.replace_time(time::Time::MIDNIGHT)
}

/// Returns the weeks including current and next 4 weeks from now
/// Each week contains start (Monday 00:00:00) and end (Sunday 23:59:59)
pub fn current_and_next_four_weeks_from_now(tz: impl Into<String>) -> [Week; 5] {
    current_and_next_four_weeks(now(tz))
}

/// Returns the weeks including current and next 4 weeks from the given date
/// Each week contains start (Monday 00:00:00) and end (Sunday 23:59:59)
pub fn current_and_next_four_weeks(from_date: OffsetDateTime) -> [Week; 5] {
    let mut weeks = [Week {
        start: OffsetDateTime::UNIX_EPOCH,
        end: OffsetDateTime::UNIX_EPOCH,
    }; 5];

    // Get the current weekday
    let current_weekday = from_date.weekday();

    // Calculate days until current week's Monday
    let days_since_monday = match current_weekday {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    };

    // Calculate the current week's Monday and reset time to 00:00:00
    let current_monday = from_date - Duration::days(days_since_monday);
    let current_monday = current_monday.replace_time(time::Time::MIDNIGHT);

    // Calculate all 5 weeks
    for i in 0..5 {
        let monday = current_monday + Duration::weeks(i as i64);
        let sunday = monday + Duration::days(6);
        let sunday_end = sunday.replace_time(time::Time::from_hms(23, 59, 59).unwrap());

        weeks[i as usize] = Week {
            start: monday,
            end: sunday_end,
        };
    }

    weeks
}

/// Returns the timestamps of the next 4 Mondays from now
/// All timestamps are set to 12:00:00
pub fn next_four_mondays_from_now(tz: impl Into<String>) -> [Week; 4] {
    next_four_mondays(now(tz))
}

/// Returns the timestamps of the next 4 Mondays from the given timestamp
/// All timestamps are set to 12:00:00
pub fn next_four_mondays(from_date: OffsetDateTime) -> [Week; 4] {
    // Get all weeks (current + next 4)
    let weeks = current_and_next_four_weeks(from_date);

    // Extract the start timestamps (Mondays) from weeks 1-4 (skip week 0 which is current)
    [weeks[1], weeks[2], weeks[3], weeks[4]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_current_and_next_four_weeks_from_monday() {
        // Start from Monday, 2025-01-20
        let monday = datetime!(2025-01-20 09:00:00 UTC);

        let result = current_and_next_four_weeks(monday);

        // Should get current week (Jan 20-26) and next 4 weeks
        let expected = [
            Week {
                start: datetime!(2025-01-20 00:00:00 UTC),
                end: datetime!(2025-01-26 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-01-27 00:00:00 UTC),
                end: datetime!(2025-02-02 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-03 00:00:00 UTC),
                end: datetime!(2025-02-09 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-10 00:00:00 UTC),
                end: datetime!(2025-02-16 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-17 00:00:00 UTC),
                end: datetime!(2025-02-23 23:59:59 UTC),
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_current_and_next_four_weeks_from_wednesday() {
        // Start from Wednesday, 2025-01-22
        let wednesday = datetime!(2025-01-22 17:00:00 UTC);

        let result = current_and_next_four_weeks(wednesday);

        // Should get current week (Jan 20-26) and next 4 weeks
        let expected = [
            Week {
                start: datetime!(2025-01-20 00:00:00 UTC),
                end: datetime!(2025-01-26 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-01-27 00:00:00 UTC),
                end: datetime!(2025-02-02 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-03 00:00:00 UTC),
                end: datetime!(2025-02-09 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-10 00:00:00 UTC),
                end: datetime!(2025-02-16 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-17 00:00:00 UTC),
                end: datetime!(2025-02-23 23:59:59 UTC),
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_current_and_next_four_weeks_from_sunday() {
        // Start from Sunday, 2025-01-26
        let sunday = datetime!(2025-01-26 22:00:00 UTC);

        let result = current_and_next_four_weeks(sunday);

        // Should get current week (Jan 20-26) and next 4 weeks
        let expected = [
            Week {
                start: datetime!(2025-01-20 00:00:00 UTC),
                end: datetime!(2025-01-26 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-01-27 00:00:00 UTC),
                end: datetime!(2025-02-02 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-03 00:00:00 UTC),
                end: datetime!(2025-02-09 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-10 00:00:00 UTC),
                end: datetime!(2025-02-16 23:59:59 UTC),
            },
            Week {
                start: datetime!(2025-02-17 00:00:00 UTC),
                end: datetime!(2025-02-23 23:59:59 UTC),
            },
        ];

        assert_eq!(result, expected);
    }
}
