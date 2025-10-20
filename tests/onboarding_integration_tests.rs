use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use sqlx::Row;
use tower::ServiceExt;

mod common;

/// Helper function to register a user and get auth cookie
async fn register_and_get_cookie(test_app: &common::TestApp) -> String {
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=test@example.com&password=password123&password_confirm=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Process events to ensure user is in read model
    test_app.process_events().await;

    // Extract auth cookie
    response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn test_post_register_shows_polling_page() {
    // AC #1: Registration shows polling page that waits for read model sync
    let (pool, _executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), _executor)).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=test@example.com&password=password123&password_confirm=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Registration now returns polling page (no immediate redirect)
    assert!(response.headers().get("ts-location").is_none());

    // Check response contains polling page
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Completing Registration"));
    assert!(body_str.contains("/register/check-user/"));
}

#[tokio::test]
async fn test_get_onboarding_renders_wizard_for_new_user() {
    // AC #1: Onboarding wizard displays for new user
    let (pool, _executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), _executor)).await;

    let cookie = register_and_get_cookie(&test_app).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/onboarding")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Verify wizard shows step 1 content (AC #2 - Dietary Restrictions)
    // Note: With TwinSpark multi-page approach, only current step is rendered
    assert!(html.contains("Dietary Restrictions"));
    assert!(html.contains("vegetarian"));
    assert!(html.contains("vegan"));
    assert!(html.contains("gluten-free"));
    assert!(html.contains("Additional Allergens"));

    // Step indicator should show 4 steps
    assert!(html.contains("step-indicator"));
}

#[tokio::test]
async fn test_get_onboarding_redirects_if_already_completed() {
    // AC #1: User who already completed onboarding gets redirected to dashboard
    let (pool, _executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), _executor)).await;

    let cookie = register_and_get_cookie(&test_app).await;

    // Complete onboarding using skip endpoint (emits all 4 step events + completion event)
    test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/onboarding/skip")
                .header("cookie", cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    test_app.process_events().await;

    // Try to access onboarding again
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/onboarding")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/dashboard");
}

#[tokio::test]
async fn test_post_onboarding_with_valid_data_creates_profile() {
    // AC #8: Completed profile stored
    // AC #9: Profile data feeds meal planning algorithm
    let (pool, _executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), _executor)).await;

    let cookie = register_and_get_cookie(&test_app).await;

    // Step 1: Submit dietary restrictions (empty for this test)
    let response1 = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/1")
                .header("cookie", cookie.clone())
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(
        response1.headers().get("ts-location").unwrap(),
        "/onboarding?step=2"
    );

    test_app.process_events().await;

    // Step 2: Submit household size
    let response2 = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/2")
                .header("cookie", cookie.clone())
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("household_size=4"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);
    assert_eq!(
        response2.headers().get("ts-location").unwrap(),
        "/onboarding?step=3"
    );

    test_app.process_events().await;

    // Step 3: Submit skill level
    let response3 = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/3")
                .header("cookie", cookie.clone())
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("skill_level=expert"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response3.status(), StatusCode::OK);
    assert_eq!(
        response3.headers().get("ts-location").unwrap(),
        "/onboarding?step=4"
    );

    test_app.process_events().await;

    // Step 4: Submit availability (defaults)
    let response4 = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/4")
                .header("cookie", cookie.clone())
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response4.status(), StatusCode::OK);
    // Step 4 now redirects to step 5 (Story 4.10: Push Notification Permission Flow)
    assert_eq!(
        response4.headers().get("ts-location").unwrap(),
        "/onboarding?step=5"
    );

    // Process events to project to read model
    test_app.process_events().await;

    // Verify profile data in database
    let user = sqlx::query(
        "SELECT dietary_restrictions, household_size, skill_level, weeknight_availability, onboarding_completed FROM users WHERE email = 'test@example.com'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let dietary_restrictions: String = user.get("dietary_restrictions");
    let household_size: i32 = user.get("household_size");
    let skill_level: String = user.get("skill_level");
    let weeknight_availability: String = user.get("weeknight_availability");
    let onboarding_completed: i32 = user.get("onboarding_completed");

    // Verify defaults and provided values
    assert_eq!(dietary_restrictions, "[]"); // Empty by default
    assert_eq!(household_size, 4);
    assert_eq!(skill_level, "expert");
    assert!(weeknight_availability.contains("18:00")); // Default start time
    assert!(weeknight_availability.contains("45")); // Default duration
                                                    // Story 4.10: Onboarding now has 5 steps, so after step 4 it's not complete
    assert_eq!(onboarding_completed, 0);
}

#[tokio::test]
async fn test_post_onboarding_applies_defaults_for_skipped_fields() {
    // AC #7: User can skip onboarding - defaults applied
    let (pool, _executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), _executor)).await;

    let cookie = register_and_get_cookie(&test_app).await;

    // Submit all steps with empty forms to apply defaults
    // Step 1: empty dietary restrictions
    test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/1")
                .header("cookie", cookie.clone())
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    test_app.process_events().await;

    // Step 2: empty household size (defaults to 2)
    test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/2")
                .header("cookie", cookie.clone())
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    test_app.process_events().await;

    // Step 3: empty skill level (defaults to intermediate)
    test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/3")
                .header("cookie", cookie.clone())
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    test_app.process_events().await;

    // Step 4: empty availability (defaults to 18:00, 45 minutes)
    test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/4")
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    // Process events
    test_app.process_events().await;

    // Verify default values were applied
    let user = sqlx::query(
        "SELECT dietary_restrictions, household_size, skill_level, weeknight_availability FROM users WHERE email = 'test@example.com'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let dietary_restrictions: String = user.get("dietary_restrictions");
    let household_size: i32 = user.get("household_size");
    let skill_level: String = user.get("skill_level");
    let weeknight_availability: String = user.get("weeknight_availability");

    // Verify defaults
    assert_eq!(dietary_restrictions, "[]"); // Empty array
    assert_eq!(household_size, 2); // Default: 2
    assert_eq!(skill_level, "intermediate"); // Default: intermediate
    assert!(weeknight_availability.contains("18:00")); // Default: 18:00
    assert!(weeknight_availability.contains("45")); // Default: 45 minutes
}

#[tokio::test]
async fn test_post_onboarding_validates_household_size_min() {
    // AC #6: Each step validates inputs before allowing progression
    let (pool, _executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), _executor)).await;

    let cookie = register_and_get_cookie(&test_app).await;

    // Submit step 2 with invalid household_size (0)
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/2")
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("household_size=0"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Validation failure redirects back to step 2
    assert_eq!(
        response.headers().get("ts-location").unwrap(),
        "/onboarding?step=2"
    );
}

#[tokio::test]
async fn test_post_onboarding_validates_household_size_max() {
    // AC #6: Each step validates inputs before allowing progression
    let (pool, _executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), _executor)).await;

    let cookie = register_and_get_cookie(&test_app).await;

    // Submit step 2 with invalid household_size (11)
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/onboarding/step/2")
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("household_size=11"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Validation failure redirects back to step 2
    assert_eq!(
        response.headers().get("ts-location").unwrap(),
        "/onboarding?step=2"
    );
}

#[tokio::test]
async fn test_get_onboarding_skip_applies_all_defaults() {
    // AC #7: User can skip onboarding (optional) - defaults applied
    let (pool, _executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), _executor)).await;

    let cookie = register_and_get_cookie(&test_app).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/onboarding/skip")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/dashboard");

    // Process events
    test_app.process_events().await;

    // Verify all defaults were applied
    let user = sqlx::query(
        "SELECT dietary_restrictions, household_size, skill_level, weeknight_availability, onboarding_completed FROM users WHERE email = 'test@example.com'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let dietary_restrictions: String = user.get("dietary_restrictions");
    let household_size: i32 = user.get("household_size");
    let skill_level: String = user.get("skill_level");
    let weeknight_availability: String = user.get("weeknight_availability");
    let onboarding_completed: i32 = user.get("onboarding_completed");

    assert_eq!(dietary_restrictions, "[]");
    assert_eq!(household_size, 2);
    assert_eq!(skill_level, "intermediate");
    assert!(weeknight_availability.contains("18:00"));
    assert!(weeknight_availability.contains("45"));
    assert_eq!(onboarding_completed, 1);
}
