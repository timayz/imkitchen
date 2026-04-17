use time::macros::format_description;
use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Weekday};
use time_tz::{PrimitiveDateTimeExt, ToTimezone, timezones};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthBounds {
    pub first: OffsetDateTime,
    pub last: OffsetDateTime,
    pub date: OffsetDateTime,
}

/// Returns the first day (00:00:00) and last day (23:59:59) of the month
/// for the given date converted to the specified timezone.
pub fn month_bounds(date: OffsetDateTime, tz: impl Into<String>) -> anyhow::Result<MonthBounds> {
    let tz = tz.into();
    let mut date = date;

    if let Some(tz) = timezones::get_by_name(&tz) {
        date = date.to_timezone(tz);
    }

    let year = date.year();
    let month = date.month();

    let first = Date::from_calendar_date(year, month, 1)?
        .with_hms(0, 0, 0)?
        .assume_offset(date.offset());

    let next_month = month.next();
    let next_year = if next_month == Month::January {
        year + 1
    } else {
        year
    };

    let last_day = Date::from_calendar_date(next_year, next_month, 1)?
        .previous_day()
        .ok_or_else(|| anyhow::anyhow!("failed to compute last day of month"))?;

    let last = last_day.with_hms(23, 59, 59)?.assume_offset(date.offset());

    Ok(MonthBounds { first, last, date })
}

/// Returns the first day (00:00:00) and last day (23:59:59) of the month
/// for a date string in "YYYY-MM-DD" format and the specified timezone.
pub fn month_bounds_from_date(date: &str, tz: impl Into<String>) -> anyhow::Result<MonthBounds> {
    let tz: String = tz.into();

    let format = format_description!("[year]-[month]-[day]");
    let date = Date::parse(date, &format)?;
    let datetime = PrimitiveDateTime::new(date, time::Time::MIDNIGHT);

    let offset_datetime = if let Some(tz_ref) = timezones::get_by_name(&tz) {
        datetime
            .assume_timezone(tz_ref)
            .map_err(|e| anyhow::anyhow!("failed to resolve timezone offset for date: {e}"))?
    } else {
        datetime.assume_utc()
    };

    month_bounds(offset_datetime, tz)
}

/// Returns the first day of the previous and next months formatted as "YYYY-MM-DD".
pub fn prev_next_month(date: OffsetDateTime) -> anyhow::Result<(String, String)> {
    let year = date.year();
    let month = date.month();

    let prev_month = month.previous();
    let prev_year = if prev_month == Month::December {
        year - 1
    } else {
        year
    };

    let next_month = month.next();
    let next_year = if next_month == Month::January {
        year + 1
    } else {
        year
    };

    let format = format_description!("[year]-[month]-[day]");

    let prev = Date::from_calendar_date(prev_year, prev_month, 1)?.format(&format)?;

    let next = Date::from_calendar_date(next_year, next_month, 1)?.format(&format)?;

    Ok((prev, next))
}

/// Returns all days of the week (Monday=0) before the given date.
/// Each day is set to 00:00:00 with the same offset as the input.
/// For example, if the date is Wednesday, returns [Monday, Tuesday].
/// If the date is Monday, returns an empty vec.
pub fn week_days_before(date: OffsetDateTime) -> Vec<OffsetDateTime> {
    let days_since_monday = match date.weekday() {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    };

    let monday = (date - Duration::days(days_since_monday)).replace_time(time::Time::MIDNIGHT);

    (0..days_since_monday)
        .map(|i| monday + Duration::days(i))
        .collect()
}

/// Returns all days of the week (Monday=0) after the given date.
/// Each day is set to 00:00:00 with the same offset as the input.
/// For example, if the date is Friday, returns [Saturday, Sunday].
/// If the date is Sunday, returns an empty vec.
pub fn week_days_after(date: OffsetDateTime) -> Vec<OffsetDateTime> {
    let days_since_monday = match date.weekday() {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    };

    let days_until_sunday = 6 - days_since_monday;
    let next_day = (date + Duration::days(1)).replace_time(time::Time::MIDNIGHT);

    (0..days_until_sunday)
        .map(|i| next_day + Duration::days(i))
        .collect()
}

pub fn month_bounds_from_now(tz: impl Into<String>) -> anyhow::Result<MonthBounds> {
    let now = OffsetDateTime::now_utc();

    month_bounds(now, tz)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Week {
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

pub fn now(tz: impl Into<String>) -> OffsetDateTime {
    let tz = tz.into();
    let mut now = OffsetDateTime::now_utc();

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
    fn test_prev_next_month_mid_year() {
        let date = datetime!(2025-06-15 10:00:00 UTC);
        let (prev, next) = prev_next_month(date).unwrap();

        assert_eq!(prev, "2025-05-01");
        assert_eq!(next, "2025-07-01");
    }

    #[test]
    fn test_prev_next_month_january() {
        let date = datetime!(2025-01-10 10:00:00 UTC);
        let (prev, next) = prev_next_month(date).unwrap();

        assert_eq!(prev, "2024-12-01");
        assert_eq!(next, "2025-02-01");
    }

    #[test]
    fn test_prev_next_month_december() {
        let date = datetime!(2025-12-25 18:00:00 UTC);
        let (prev, next) = prev_next_month(date).unwrap();

        assert_eq!(prev, "2025-11-01");
        assert_eq!(next, "2026-01-01");
    }

    #[test]
    fn test_week_days_before_wednesday() {
        let wed = datetime!(2025-01-22 14:30:00 UTC);
        let result = week_days_before(wed);

        assert_eq!(
            result,
            vec![
                datetime!(2025-01-20 00:00:00 UTC), // Monday
                datetime!(2025-01-21 00:00:00 UTC), // Tuesday
            ]
        );
    }

    #[test]
    fn test_week_days_before_monday() {
        let mon = datetime!(2025-01-20 09:00:00 UTC);
        let result = week_days_before(mon);

        assert!(result.is_empty());
    }

    #[test]
    fn test_week_days_before_sunday() {
        let sun = datetime!(2025-01-26 22:00:00 UTC);
        let result = week_days_before(sun);

        assert_eq!(
            result,
            vec![
                datetime!(2025-01-20 00:00:00 UTC), // Monday
                datetime!(2025-01-21 00:00:00 UTC), // Tuesday
                datetime!(2025-01-22 00:00:00 UTC), // Wednesday
                datetime!(2025-01-23 00:00:00 UTC), // Thursday
                datetime!(2025-01-24 00:00:00 UTC), // Friday
                datetime!(2025-01-25 00:00:00 UTC), // Saturday
            ]
        );
    }

    #[test]
    fn test_week_days_after_friday() {
        let fri = datetime!(2025-01-24 14:30:00 UTC);
        let result = week_days_after(fri);

        assert_eq!(
            result,
            vec![
                datetime!(2025-01-25 00:00:00 UTC), // Saturday
                datetime!(2025-01-26 00:00:00 UTC), // Sunday
            ]
        );
    }

    #[test]
    fn test_week_days_after_sunday() {
        let sun = datetime!(2025-01-26 22:00:00 UTC);
        let result = week_days_after(sun);

        assert!(result.is_empty());
    }

    #[test]
    fn test_week_days_after_monday() {
        let mon = datetime!(2025-01-20 09:00:00 UTC);
        let result = week_days_after(mon);

        assert_eq!(
            result,
            vec![
                datetime!(2025-01-21 00:00:00 UTC), // Tuesday
                datetime!(2025-01-22 00:00:00 UTC), // Wednesday
                datetime!(2025-01-23 00:00:00 UTC), // Thursday
                datetime!(2025-01-24 00:00:00 UTC), // Friday
                datetime!(2025-01-25 00:00:00 UTC), // Saturday
                datetime!(2025-01-26 00:00:00 UTC), // Sunday
            ]
        );
    }

    #[test]
    fn test_month_bounds_january() {
        let date = datetime!(2025-01-15 10:30:00 UTC);
        let result = month_bounds(date, "UTC").unwrap();

        assert_eq!(
            result,
            MonthBounds {
                first: datetime!(2025-01-01 00:00:00 UTC),
                last: datetime!(2025-01-31 23:59:59 UTC),
            }
        );
    }

    #[test]
    fn test_month_bounds_february_non_leap() {
        let date = datetime!(2025-02-20 14:00:00 UTC);
        let result = month_bounds(date, "UTC").unwrap();

        assert_eq!(
            result,
            MonthBounds {
                first: datetime!(2025-02-01 00:00:00 UTC),
                last: datetime!(2025-02-28 23:59:59 UTC),
            }
        );
    }

    #[test]
    fn test_month_bounds_february_leap() {
        let date = datetime!(2024-02-10 08:00:00 UTC);
        let result = month_bounds(date, "UTC").unwrap();

        assert_eq!(
            result,
            MonthBounds {
                first: datetime!(2024-02-01 00:00:00 UTC),
                last: datetime!(2024-02-29 23:59:59 UTC),
            }
        );
    }

    #[test]
    fn test_month_bounds_december() {
        let date = datetime!(2025-12-25 18:00:00 UTC);
        let result = month_bounds(date, "UTC").unwrap();

        assert_eq!(
            result,
            MonthBounds {
                first: datetime!(2025-12-01 00:00:00 UTC),
                last: datetime!(2025-12-31 23:59:59 UTC),
            }
        );
    }

    #[test]
    fn test_month_bounds_from_date() {
        let result = month_bounds_from_date("2025-03-15", "UTC").unwrap();

        assert_eq!(
            result,
            MonthBounds {
                first: datetime!(2025-03-01 00:00:00 UTC),
                last: datetime!(2025-03-31 23:59:59 UTC),
            }
        );
    }

    #[test]
    fn test_month_bounds_from_date_invalid() {
        let result = month_bounds_from_date("not-a-date", "UTC");
        assert!(result.is_err());
    }

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
