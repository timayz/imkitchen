//! The skeleton of the app, driven without a screen: its layout for the room
//! a window gives, its navigation and its languages.

use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementQuery};
use imkitchen_app::{AppWindow, Form, Navigation, Screen};
use slint::{ComponentHandle, LogicalSize};

/// The app in a window of that size, in English whatever the machine speaks.
fn app(width: f32, height: f32) -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let app = imkitchen_app::window().unwrap();
    slint::select_bundled_translation("en").unwrap();
    app.window().set_size(LogicalSize::new(width, height));
    app.show().unwrap();
    app
}

fn bar(app: &AppWindow) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_accessible_role(AccessibleRole::TabList)
        .find_first()
        .expect("the navigation bar")
}

fn tabs(app: &AppWindow) -> Vec<ElementHandle> {
    ElementQuery::from_root(app)
        .match_accessible_role(AccessibleRole::Tab)
        .find_all()
}

fn labels(app: &AppWindow) -> Vec<String> {
    tabs(app)
        .iter()
        .map(|tab| tab.accessible_label().unwrap().to_string())
        .collect()
}

fn shows(app: &AppWindow, text: &str) -> bool {
    ElementHandle::find_by_accessible_label(app, text)
        .next()
        .is_some()
}

#[test]
fn a_phone_has_its_navigation_along_the_bottom() {
    let app = app(390.0, 844.0);
    assert_eq!(app.get_form(), Form::Compact);

    let bar = bar(&app);
    let (position, size) = (bar.absolute_position(), bar.size());
    assert_eq!(position.x, 0.0);
    assert_eq!(size.width, 390.0);
    assert_eq!(position.y + size.height, 844.0);
    assert_eq!(size.height, 80.0);

    // The screen has the rest of the window.
    let text = ElementHandle::find_by_accessible_label(
        &app,
        "The recipes you write or import are kept here.",
    )
    .next()
    .unwrap();
    assert_eq!(text.absolute_position().x, 16.0);
    assert_eq!(text.size().width, 390.0 - 2.0 * 16.0);
    assert!(text.absolute_position().y < position.y);
}

#[test]
fn a_wider_window_has_its_navigation_down_the_side() {
    let medium = app(700.0, 900.0);
    assert_eq!(medium.get_form(), Form::Medium);
    let rail = bar(&medium).size();
    assert_eq!(bar(&medium).absolute_position().y, 0.0);
    assert_eq!(rail.height, 900.0);
    assert_eq!(rail.width, 96.0);

    // The same window, pulled wider: the labels move beside the icons.
    medium.window().set_size(LogicalSize::new(1280.0, 800.0));
    assert_eq!(medium.get_form(), Form::Expanded);
    let side = bar(&medium).size();
    assert_eq!(side.height, 800.0);
    assert_eq!(side.width, 240.0);

    // The screen is a column in the middle of what the bar leaves.
    let text = ElementHandle::find_by_accessible_label(
        &medium,
        "The recipes you write or import are kept here.",
    )
    .next()
    .unwrap();
    assert_eq!(text.size().width, 840.0 - 2.0 * 32.0);
    assert_eq!(
        text.absolute_position().x,
        240.0 + (1280.0 - 240.0 - 840.0) / 2.0 + 32.0
    );
}

#[test]
fn every_destination_is_a_tap_away() {
    let app = app(390.0, 844.0);
    assert_eq!(
        labels(&app),
        ["Recipes", "Dishes", "List", "Groceries", "Settings"]
    );
    assert_eq!(app.global::<Navigation>().get_current(), Screen::Recipes);
    assert!(shows(
        &app,
        "The recipes you write or import are kept here."
    ));

    let screens = [
        Screen::Recipes,
        Screen::Dishes,
        Screen::List,
        Screen::Groceries,
        Screen::Settings,
    ];
    for (tab, screen) in tabs(&app).iter().zip(screens).rev() {
        tab.invoke_accessible_default_action();
        assert_eq!(app.global::<Navigation>().get_current(), screen);
        assert_eq!(tab.accessible_item_selected(), Some(true));
    }

    tabs(&app)[1].invoke_accessible_default_action();
    assert!(shows(
        &app,
        "The dishes you compose with your recipes are kept here."
    ));
    assert!(!shows(
        &app,
        "The recipes you write or import are kept here."
    ));
}

#[test]
fn the_app_tells_its_version() {
    let app = app(1280.0, 800.0);
    app.global::<Navigation>().set_current(Screen::Settings);
    assert!(shows(
        &app,
        &format!("imkitchen {}", env!("CARGO_PKG_VERSION"))
    ));
}

#[test]
fn the_app_speaks_french() {
    let app = app(390.0, 844.0);
    slint::select_bundled_translation("fr").unwrap();
    assert_eq!(
        labels(&app),
        ["Recettes", "Plats", "Liste", "Courses", "Réglages"]
    );
    assert!(shows(
        &app,
        "Les recettes que vous écrivez ou importez sont gardées ici."
    ));
}
