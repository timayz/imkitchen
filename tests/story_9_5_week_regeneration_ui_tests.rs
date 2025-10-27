/// Integration tests for Story 9.5: Week Regeneration UI with Confirmation
///
/// These tests verify that the template files and JavaScript exist with correct structure:
/// - AC-9.5.1: "Regenerate This Week" button template
/// - AC-9.5.2: "Regenerate All Future Weeks" button template
/// - AC-9.5.3-9.5.5: Modal component structure
/// - AC-9.5.7: Loading spinner component
/// - AC-9.5.10: Locked week disabled text
///
/// Note: Full integration tests with route handlers are in week_regeneration_integration_tests.rs
use std::fs;

/// AC-9.5.1, AC-9.5.2, AC-9.5.10: Verify multi_week_calendar.html has regeneration buttons
#[test]
fn test_multi_week_calendar_has_regeneration_buttons() {
    let template_path = "templates/meal_plan/multi_week_calendar.html";
    let content = fs::read_to_string(template_path)
        .expect("Failed to read multi_week_calendar.html template");

    // AC-9.5.2: "Regenerate All Future Weeks" button
    assert!(
        content.contains("openRegenerateAllModal"),
        "Template should contain openRegenerateAllModal function call"
    );
    assert!(
        content.contains("Regenerate All Future Weeks"),
        "Template should contain 'Regenerate All Future Weeks' button text"
    );
    assert!(
        content.contains("bg-orange-600"),
        "Regenerate All button should have warning color (orange)"
    );

    // AC-9.5.1: "Regenerate This Week" button
    assert!(
        content.contains("openRegenerateWeekModal"),
        "Template should contain openRegenerateWeekModal function call"
    );
    assert!(
        content.contains("Regenerate This Week"),
        "Template should contain 'Regenerate This Week' button text"
    );
    assert!(
        content.contains("bg-yellow-500"),
        "Regenerate This Week button should have yellow styling"
    );

    // AC-9.5.10: Locked weeks show disabled text
    assert!(
        content.contains("Cannot Regenerate (week in progress)"),
        "Template should contain disabled text for locked weeks"
    );
    assert!(
        content.contains("{% if not week.is_locked %}"),
        "Template should conditionally render button based on is_locked"
    );
}

/// AC-9.5.3, AC-9.5.4, AC-9.5.5: Verify week-regeneration-modal.html exists with correct structure
#[test]
fn test_week_regeneration_modal_structure() {
    let modal_path = "templates/components/week-regeneration-modal.html";
    let content =
        fs::read_to_string(modal_path).expect("Failed to read week-regeneration-modal.html");

    // Modal container
    assert!(
        content.contains("id=\"week-regeneration-modal\""),
        "Modal should have ID 'week-regeneration-modal'"
    );
    assert!(
        content.contains("role=\"dialog\""),
        "Modal should have role='dialog' for accessibility"
    );
    assert!(
        content.contains("aria-modal=\"true\""),
        "Modal should have aria-modal='true'"
    );

    // AC-9.5.3, AC-9.5.4: Modal title
    assert!(
        content.contains("Confirm Regeneration"),
        "Modal should have 'Confirm Regeneration' title"
    );

    // AC-9.5.5: Cancel button (secondary styling)
    assert!(
        content.contains("data-close-week-regeneration-modal"),
        "Modal should have close button with data attribute"
    );
    assert!(
        content.contains("bg-gray-200"),
        "Cancel button should have secondary styling (gray)"
    );
    assert!(
        content.contains("Cancel"),
        "Modal should have 'Cancel' button text"
    );

    // AC-9.5.5: Confirm button (primary/danger styling)
    assert!(
        content.contains("id=\"confirm-regeneration-btn\""),
        "Modal should have Confirm button with ID"
    );
    assert!(
        content.contains("bg-red-600"),
        "Confirm button should have danger styling (red)"
    );
    assert!(
        content.contains("Confirm"),
        "Modal should have 'Confirm' button text"
    );

    // Hidden inputs for JavaScript
    assert!(
        content.contains("id=\"regeneration-type\""),
        "Modal should have hidden input for regeneration type"
    );
    assert!(
        content.contains("id=\"regeneration-week-id\""),
        "Modal should have hidden input for week ID"
    );
}

/// AC-9.5.7: Verify loading spinner exists
#[test]
fn test_loading_spinner_exists() {
    let modal_path = "templates/components/week-regeneration-modal.html";
    let content =
        fs::read_to_string(modal_path).expect("Failed to read week-regeneration-modal.html");

    // AC-9.5.7: Loading spinner overlay
    assert!(
        content.contains("id=\"regeneration-loading-spinner\""),
        "Template should contain loading spinner overlay"
    );
    assert!(
        content.contains("Regenerating meals"),
        "Spinner should have 'Regenerating meals' text"
    );
    assert!(
        content.contains("animate-spin"),
        "Spinner should have animation class"
    );
}

/// Verify week-regeneration.js exists and has required functions
#[test]
fn test_week_regeneration_js_exists() {
    let js_path = "static/js/week-regeneration.js";
    let content = fs::read_to_string(js_path).expect("Failed to read week-regeneration.js");

    // AC-9.5.3: openRegenerateWeekModal function
    assert!(
        content.contains("window.openRegenerateWeekModal = function"),
        "JS should define openRegenerateWeekModal function"
    );

    // AC-9.5.4: openRegenerateAllModal function
    assert!(
        content.contains("window.openRegenerateAllModal = function"),
        "JS should define openRegenerateAllModal function"
    );

    // AC-9.5.7: showSpinner function
    assert!(
        content.contains("function showSpinner()"),
        "JS should define showSpinner function"
    );
    assert!(
        content.contains("function hideSpinner()"),
        "JS should define hideSpinner function"
    );

    // AC-9.5.9: showErrorToast function
    assert!(
        content.contains("function showErrorToast(message)"),
        "JS should define showErrorToast function"
    );
    assert!(
        content.contains("Failed to regenerate. Please try again."),
        "JS should have error message text"
    );

    // AC-9.5.6: POST request handling
    assert!(
        content.contains("fetch(url, {"),
        "JS should use fetch for POST requests"
    );
    assert!(
        content.contains("method: 'POST'"),
        "JS should send POST requests"
    );

    // Keyboard navigation
    assert!(
        content.contains("event.key === 'Escape'"),
        "JS should handle Escape key to close modal"
    );
    assert!(
        content.contains("event.key === 'Tab'"),
        "JS should handle Tab key for focus trap"
    );
}

/// Verify JavaScript is loaded in multi_week_calendar.html
#[test]
fn test_multi_week_calendar_loads_regeneration_js() {
    let template_path = "templates/meal_plan/multi_week_calendar.html";
    let content =
        fs::read_to_string(template_path).expect("Failed to read multi_week_calendar.html");

    assert!(
        content.contains("week-regeneration.js"),
        "Template should load week-regeneration.js script"
    );
}

/// Verify modal is included in multi_week_calendar.html
#[test]
fn test_multi_week_calendar_includes_modal() {
    let template_path = "templates/meal_plan/multi_week_calendar.html";
    let content =
        fs::read_to_string(template_path).expect("Failed to read multi_week_calendar.html");

    assert!(
        content.contains("{% include \"components/week-regeneration-modal.html\" %}"),
        "Template should include week-regeneration-modal component"
    );
}

/// Test that template compilation succeeds (verified at build time by Askama)
#[test]
fn test_templates_compile() {
    // This test verifies that all templates compile successfully
    // Askama performs compile-time template checking
    // If this test compiles, all templates are valid
    // No runtime assertion needed - verified at build time
}
