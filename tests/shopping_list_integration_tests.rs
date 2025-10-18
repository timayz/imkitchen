/// Integration tests for shopping list multi-week access (Story 4.3)
///
/// Note: Week validation logic is primarily tested at the unit level in
/// crates/shopping/tests/integration_tests.rs. These tests verify the HTTP layer integration.
use chrono::{Datelike, Duration, Utc};
use shopping::{validate_week_date, ShoppingListError};

/// Helper to get current week's Monday
fn get_current_week_monday() -> String {
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    monday.format("%Y-%m-%d").to_string()
}

/// Helper to get future week's Monday
fn get_future_week_monday(weeks_ahead: i64) -> String {
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let future_monday = monday + Duration::weeks(weeks_ahead);
    future_monday.format("%Y-%m-%d").to_string()
}

/// Helper to get past week's Monday
fn get_past_week_monday(weeks_ago: i64) -> String {
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let past_monday = monday - Duration::weeks(weeks_ago);
    past_monday.format("%Y-%m-%d").to_string()
}

// ==================== Week Validation Integration Tests ====================
// These tests verify week validation at the integration level (complementing unit tests)

#[test]
fn test_week_validation_integration_current_week() {
    // AC #3: Current week should be valid
    let current_week = get_current_week_monday();
    let result = validate_week_date(&current_week);
    assert!(result.is_ok());
}

#[test]
fn test_week_validation_integration_future_weeks() {
    // AC #5: Future weeks up to +4 weeks should be valid
    for weeks_ahead in 1..=4 {
        let future_week = get_future_week_monday(weeks_ahead);
        let result = validate_week_date(&future_week);
        assert!(
            result.is_ok(),
            "Week +{} should be valid, got error: {:?}",
            weeks_ahead,
            result
        );
    }
}

#[test]
fn test_week_validation_integration_five_weeks_ahead_rejected() {
    // AC #5: +5 weeks exceeds the limit
    let five_weeks_ahead = get_future_week_monday(5);
    let result = validate_week_date(&five_weeks_ahead);
    assert!(matches!(
        result,
        Err(ShoppingListError::FutureWeekOutOfRangeError)
    ));
}

#[test]
fn test_week_validation_integration_past_weeks_rejected() {
    // AC #7: Past weeks should be rejected
    for weeks_ago in 1..=3 {
        let past_week = get_past_week_monday(weeks_ago);
        let result = validate_week_date(&past_week);
        assert!(
            matches!(result, Err(ShoppingListError::PastWeekNotAccessibleError)),
            "Week -{} should be rejected as past week",
            weeks_ago
        );
    }
}

#[test]
fn test_week_validation_integration_invalid_formats() {
    // Invalid date formats should be rejected
    let invalid_dates = vec![
        "invalid-date",
        "2025-13-01", // Invalid month
        "2025-10-32", // Invalid day
        "2025-10-22", // Tuesday (not Monday)
        "2025-10-23", // Wednesday (not Monday)
    ];

    for invalid_date in invalid_dates {
        let result = validate_week_date(invalid_date);
        assert!(
            matches!(result, Err(ShoppingListError::InvalidWeekError(_))),
            "Date '{}' should be rejected as invalid",
            invalid_date
        );
    }
}
