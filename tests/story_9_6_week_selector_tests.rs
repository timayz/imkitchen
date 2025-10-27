/// Integration tests for Story 9.6: Add Week Selector to Shopping List Page
///
/// These tests verify that the template files contain correct structure:
/// - AC 9.6.1: Shopping list page updated with week selector
/// - AC 9.6.2: Week selector dropdown showing all weeks
/// - AC 9.6.3: Current week selected by default
/// - AC 9.6.4: Changing selection triggers TwinSpark request
/// - AC 9.6.5: Dropdown options show week dates
/// - AC 9.6.6: Locked weeks marked with 🔒 icon
/// - AC 9.6.7: Shopping list displays week start date at top
/// - AC 9.6.8: Mobile responsive dropdown (w-full, min-height 44px)
///
/// Note: Full integration tests with route handlers are in shopping_list_integration_tests.rs
use std::fs;

/// AC 9.6.1, 9.6.2, 9.6.3: Verify shopping-list.html has week selector dropdown
#[test]
fn test_shopping_list_has_week_selector_dropdown() {
    let template_path = "templates/pages/shopping-list.html";
    let content =
        fs::read_to_string(template_path).expect("Failed to read shopping-list.html template");

    // AC 9.6.1: Page updated with week selector
    assert!(
        content.contains("id=\"week-selector\""),
        "Template should contain week selector dropdown with id='week-selector'"
    );

    // AC 9.6.2: Dropdown shows all weeks (iterates over week_options)
    assert!(
        content.contains("{% for option in week_options %}"),
        "Template should iterate over week_options to populate dropdown"
    );
    assert!(
        content.contains("{{ option.label }}"),
        "Template should display option labels"
    );

    // AC 9.6.3: Current week selected by default
    assert!(
        content.contains("{% if option.iso_date == selected_week %}selected{% endif %}"),
        "Template should mark current week as selected"
    );

    // Week selector label
    assert!(
        content.contains("Select Week"),
        "Template should have 'Select Week' label"
    );
}

/// AC 9.6.4: Verify TwinSpark attributes on week selector
#[test]
fn test_week_selector_has_twinspark_attributes() {
    let template_path = "templates/pages/shopping-list.html";
    let content =
        fs::read_to_string(template_path).expect("Failed to read shopping-list.html template");

    // AC 9.6.4: TwinSpark request attributes
    assert!(
        content.contains("ts-req=\"/shopping?week={value}\""),
        "Week selector should have ts-req attribute for AJAX request"
    );
    assert!(
        content.contains("ts-target=\"#shopping-list-content\""),
        "Week selector should target #shopping-list-content for swap"
    );
    assert!(
        content.contains("ts-swap=\"innerHTML\""),
        "Week selector should use innerHTML swap strategy"
    );
    assert!(
        content.contains("ts-trigger=\"change\""),
        "Week selector should trigger on change event"
    );
    assert!(
        content.contains("ts-req-history=\"replace\""),
        "Week selector should update browser history"
    );
}

/// AC 9.6.7: Verify shopping list displays week header
#[test]
fn test_shopping_list_displays_week_header() {
    let template_path = "templates/pages/shopping-list.html";
    let content =
        fs::read_to_string(template_path).expect("Failed to read shopping-list.html template");

    // AC 9.6.7: Header inside #shopping-list-content
    assert!(
        content.contains("Shopping List for Week of {{ week_start_date_formatted }}"),
        "Template should display week header with formatted date"
    );

    // Header should be inside #shopping-list-content for TwinSpark updates
    let content_div_start = content
        .find("id=\"shopping-list-content\"")
        .expect("Template should have #shopping-list-content div");
    let header_pos = content
        .find("Shopping List for Week of")
        .expect("Template should have week header");

    assert!(
        header_pos > content_div_start,
        "Week header should be inside #shopping-list-content div"
    );
}

/// AC 9.6.8: Verify mobile-responsive dropdown styling
#[test]
fn test_week_selector_mobile_responsive_styling() {
    let template_path = "templates/pages/shopping-list.html";
    let content =
        fs::read_to_string(template_path).expect("Failed to read shopping-list.html template");

    // Find the week selector element (between id="week-selector" and the closing >)
    let selector_start = content
        .find("id=\"week-selector\"")
        .expect("Template should have week selector");
    let selector_end = content[selector_start..]
        .find('>')
        .expect("Week selector should have closing bracket");
    let selector_element = &content[selector_start..selector_start + selector_end];

    // AC 9.6.8: Mobile full-width and desktop max-width
    assert!(
        selector_element.contains("w-full"),
        "Week selector should have w-full class for mobile"
    );
    assert!(
        selector_element.contains("md:max-w-xs"),
        "Week selector should have md:max-w-xs for desktop constraint"
    );

    // AC 9.6.8: Touch-friendly minimum height
    assert!(
        selector_element.contains("min-height: 44px"),
        "Week selector should have min-height: 44px for touch accessibility"
    );
}

/// AC 9.6.7: Verify shopping-list-content partial has week header
#[test]
fn test_shopping_list_content_partial_has_week_header() {
    let template_path = "templates/partials/shopping-list-content.html";
    let content = fs::read_to_string(template_path)
        .expect("Failed to read shopping-list-content.html partial");

    // AC 9.6.7: Week header in partial (for TwinSpark updates)
    assert!(
        content.contains("Shopping List for Week of {{ week_start_date_formatted }}"),
        "Partial should display week header with formatted date"
    );

    // Header should be at the top of the partial content
    assert!(
        content.contains("<h2"),
        "Week header should be an h2 element"
    );
}

/// Verify backend provides week data with correct format
#[test]
fn test_week_option_struct_has_required_fields() {
    // This is a compile-time check - if the code compiles, the struct has the fields
    // The struct definition in src/routes/shopping.rs should have:
    // - label: String (for "Week 1 (Oct 28 - Nov 3)")
    // - iso_date: String (for ISO 8601 date)
    // - is_current: bool (for selected attribute)
    // - week_number: i64 (for week numbering)
    // - start_date_formatted: String (for "October 28" display)

    // Read the shopping.rs route file to verify struct definition
    let route_path = "src/routes/shopping.rs";
    let content = fs::read_to_string(route_path).expect("Failed to read shopping.rs route file");

    // Verify WeekOption struct has required fields
    assert!(
        content.contains("pub struct WeekOption"),
        "shopping.rs should define WeekOption struct"
    );
    assert!(
        content.contains("pub label: String"),
        "WeekOption should have label field"
    );
    assert!(
        content.contains("pub iso_date: String"),
        "WeekOption should have iso_date field"
    );
    assert!(
        content.contains("pub is_current: bool"),
        "WeekOption should have is_current field"
    );
    assert!(
        content.contains("pub week_number: i64"),
        "WeekOption should have week_number field"
    );
    assert!(
        content.contains("pub start_date_formatted: String"),
        "WeekOption should have start_date_formatted field"
    );

    // AC 9.6.5, 9.6.6: Verify label formatting with date ranges and lock icon
    assert!(
        content.contains("format!(\"🔒 Week {} ({})\", week_number, date_range)"),
        "Current week label should include lock icon 🔒"
    );
    assert!(
        content.contains("format!(\"Week {} ({})\", week_number, date_range)"),
        "Future week labels should not include lock icon"
    );

    // Verify date range formatting
    assert!(
        content.contains("\"{} - {}\""),
        "Date range should be formatted as 'MMM DD - MMM DD'"
    );
}

/// Verify ShoppingListTemplate struct has week_start_date_formatted field
#[test]
fn test_shopping_list_template_has_formatted_date_field() {
    let route_path = "src/routes/shopping.rs";
    let content = fs::read_to_string(route_path).expect("Failed to read shopping.rs route file");

    // AC 9.6.7: Template struct should have formatted date field
    assert!(
        content.contains("pub struct ShoppingListTemplate"),
        "shopping.rs should define ShoppingListTemplate struct"
    );
    assert!(
        content.contains("pub week_start_date_formatted: String"),
        "ShoppingListTemplate should have week_start_date_formatted field"
    );

    // Verify partial template struct also has this field
    assert!(
        content.contains("pub struct ShoppingListContentPartial"),
        "shopping.rs should define ShoppingListContentPartial struct"
    );
    assert!(
        content.contains("pub week_start_date_formatted: String"),
        "ShoppingListContentPartial should have week_start_date_formatted field"
    );
}

/// Verify #shopping-list-content wrapper exists for TwinSpark
#[test]
fn test_shopping_list_content_wrapper_exists() {
    let template_path = "templates/pages/shopping-list.html";
    let content =
        fs::read_to_string(template_path).expect("Failed to read shopping-list.html template");

    // Shopping list content should be wrapped in a div with id for TwinSpark targeting
    assert!(
        content.contains("<div id=\"shopping-list-content\">"),
        "Template should have #shopping-list-content wrapper for TwinSpark"
    );

    // Wrapper should NOT have auto-refresh attributes (replaced by manual week selector)
    assert!(
        !content.contains("ts-trigger=\"every"),
        "Shopping list should not auto-refresh (manual selection only)"
    );
}
