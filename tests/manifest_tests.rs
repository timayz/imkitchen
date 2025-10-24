use serde_json::Value;

#[test]
fn test_manifest_json_valid_schema() {
    let manifest_content = include_str!("../static/manifest.json");
    let manifest: Value =
        serde_json::from_str(manifest_content).expect("manifest.json should be valid JSON");

    // Verify required fields exist
    assert!(
        manifest.get("name").is_some(),
        "manifest must have 'name' field"
    );
    assert!(
        manifest.get("short_name").is_some(),
        "manifest must have 'short_name' field"
    );
    assert!(
        manifest.get("description").is_some(),
        "manifest must have 'description' field"
    );
    assert!(
        manifest.get("start_url").is_some(),
        "manifest must have 'start_url' field"
    );
    assert!(
        manifest.get("display").is_some(),
        "manifest must have 'display' field"
    );
    assert!(
        manifest.get("theme_color").is_some(),
        "manifest must have 'theme_color' field"
    );
    assert!(
        manifest.get("background_color").is_some(),
        "manifest must have 'background_color' field"
    );
    assert!(
        manifest.get("icons").is_some(),
        "manifest must have 'icons' array"
    );

    // Verify field values match requirements
    assert_eq!(
        manifest["name"].as_str().unwrap(),
        "imkitchen - Intelligent Meal Planning",
        "name must match expected value"
    );
    assert_eq!(
        manifest["short_name"].as_str().unwrap(),
        "imkitchen",
        "short_name must match expected value"
    );
    assert_eq!(
        manifest["description"].as_str().unwrap(),
        "Automated meal planning and cooking optimization",
        "description must match expected value"
    );
    assert_eq!(
        manifest["start_url"].as_str().unwrap(),
        "/",
        "start_url must be /"
    );
    assert_eq!(
        manifest["display"].as_str().unwrap(),
        "standalone",
        "display must be standalone"
    );
    assert_eq!(
        manifest["theme_color"].as_str().unwrap(),
        "#2563eb",
        "theme_color must be #2563eb"
    );
    assert_eq!(
        manifest["background_color"].as_str().unwrap(),
        "#ffffff",
        "background_color must be #ffffff"
    );
    assert_eq!(
        manifest["orientation"].as_str().unwrap(),
        "portrait-primary",
        "orientation must be portrait-primary"
    );
    assert_eq!(manifest["scope"].as_str().unwrap(), "/", "scope must be /");
}

#[test]
fn test_manifest_icons_array() {
    let manifest_content = include_str!("../static/manifest.json");
    let manifest: Value =
        serde_json::from_str(manifest_content).expect("manifest.json should be valid JSON");

    let icons = manifest["icons"]
        .as_array()
        .expect("icons should be an array");

    assert!(!icons.is_empty(), "icons array must not be empty");

    // Verify at least one 192x192 icon
    let has_192 = icons
        .iter()
        .any(|icon| icon["sizes"].as_str().unwrap_or("") == "192x192");
    assert!(has_192, "manifest must include 192x192 icon");

    // Verify at least one 512x512 icon
    let has_512 = icons
        .iter()
        .any(|icon| icon["sizes"].as_str().unwrap_or("") == "512x512");
    assert!(has_512, "manifest must include 512x512 icon");

    // Verify at least one maskable icon
    let has_maskable = icons
        .iter()
        .any(|icon| icon["purpose"].as_str().unwrap_or("") == "maskable");
    assert!(
        has_maskable,
        "manifest must include at least one maskable icon"
    );

    // Verify all icons have required fields
    for icon in icons {
        assert!(icon.get("src").is_some(), "icon must have 'src' field");
        assert!(icon.get("sizes").is_some(), "icon must have 'sizes' field");
        assert!(icon.get("type").is_some(), "icon must have 'type' field");
        assert!(
            icon.get("purpose").is_some(),
            "icon must have 'purpose' field"
        );
    }
}

#[test]
fn test_manifest_screenshots_array() {
    let manifest_content = include_str!("../static/manifest.json");
    let manifest: Value =
        serde_json::from_str(manifest_content).expect("manifest.json should be valid JSON");

    let screenshots = manifest["screenshots"]
        .as_array()
        .expect("screenshots should be an array");

    assert!(
        !screenshots.is_empty(),
        "screenshots array must not be empty"
    );

    // Verify all screenshots have required fields
    for screenshot in screenshots {
        assert!(
            screenshot.get("src").is_some(),
            "screenshot must have 'src' field"
        );
        assert!(
            screenshot.get("sizes").is_some(),
            "screenshot must have 'sizes' field"
        );
        assert!(
            screenshot.get("type").is_some(),
            "screenshot must have 'type' field"
        );
        assert!(
            screenshot.get("platform").is_some(),
            "screenshot must have 'platform' field"
        );
    }
}

#[test]
fn test_manifest_shortcuts_array() {
    let manifest_content = include_str!("../static/manifest.json");
    let manifest: Value =
        serde_json::from_str(manifest_content).expect("manifest.json should be valid JSON");

    let shortcuts = manifest["shortcuts"]
        .as_array()
        .expect("shortcuts should be an array");

    assert!(!shortcuts.is_empty(), "shortcuts array must not be empty");

    // Verify "Today's Meals" shortcut exists
    let has_dashboard = shortcuts.iter().any(|shortcut| {
        shortcut["name"].as_str().unwrap_or("") == "Today's Meals"
            && shortcut["url"].as_str().unwrap_or("") == "/"
    });
    assert!(
        has_dashboard,
        "manifest must include 'Today's Meals' shortcut"
    );

    // Verify "Recipes" shortcut exists
    let has_recipes = shortcuts.iter().any(|shortcut| {
        shortcut["name"].as_str().unwrap_or("") == "Recipes"
            && shortcut["url"].as_str().unwrap_or("") == "/recipes"
    });
    assert!(has_recipes, "manifest must include 'Recipes' shortcut");

    // Verify all shortcuts have required fields
    for shortcut in shortcuts {
        assert!(
            shortcut.get("name").is_some(),
            "shortcut must have 'name' field"
        );
        assert!(
            shortcut.get("url").is_some(),
            "shortcut must have 'url' field"
        );
        assert!(
            shortcut.get("icons").is_some(),
            "shortcut must have 'icons' array"
        );
    }
}

#[test]
fn test_manifest_categories() {
    let manifest_content = include_str!("../static/manifest.json");
    let manifest: Value =
        serde_json::from_str(manifest_content).expect("manifest.json should be valid JSON");

    let categories = manifest["categories"]
        .as_array()
        .expect("categories should be an array");

    assert!(
        categories.contains(&Value::String("lifestyle".to_string())),
        "categories must include 'lifestyle'"
    );
    assert!(
        categories.contains(&Value::String("food".to_string())),
        "categories must include 'food'"
    );
}
