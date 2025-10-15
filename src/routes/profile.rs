use askama::Template;
use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde_json::json;
use sqlx::Row;

use crate::middleware::Auth;
use crate::routes::auth::AppState;

#[derive(Template)]
#[template(path = "pages/onboarding.html")]
pub struct OnboardingPageTemplate {
    pub error: String,
    pub user: Option<()>,
    pub current_step: u8,
    pub dietary_restrictions: Vec<String>,
    pub allergens: String,
    pub household_size: String,
    pub skill_level: String,
    pub availability_start: String,
    pub availability_duration: String,
}

#[derive(Debug, Default)]
pub struct OnboardingForm {
    pub dietary_restrictions: Vec<String>,
    pub allergens: String,
    pub household_size: String,
    pub skill_level: String,
    pub availability_start: String,
    pub availability_duration: String,
}

impl OnboardingForm {
    fn from_form_data(data: &str) -> Self {
        let mut form = OnboardingForm::default();

        for pair in data.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key).unwrap_or_default();
                let value = urlencoding::decode(value).unwrap_or_default();

                match key.as_ref() {
                    "dietary_restrictions" => {
                        form.dietary_restrictions.push(value.to_string());
                    }
                    "allergens" => {
                        form.allergens = value.to_string();
                    }
                    "household_size" => {
                        form.household_size = value.to_string();
                    }
                    "skill_level" => {
                        form.skill_level = value.to_string();
                    }
                    "availability_start" => {
                        form.availability_start = value.to_string();
                    }
                    "availability_duration" => {
                        form.availability_duration = value.to_string();
                    }
                    _ => {}
                }
            }
        }

        form
    }
}

use axum::extract::Query;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StepQuery {
    step: Option<u8>,
}

/// GET /onboarding - Display onboarding wizard
///
/// AC #1: Onboarding wizard displays after first registration
/// Checks if user has already completed onboarding and redirects to dashboard if true
#[tracing::instrument(skip(state, auth))]
pub async fn get_onboarding(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Query(query): Query<StepQuery>,
) -> Response {
    // Check if user has already completed onboarding
    let user_query = sqlx::query("SELECT onboarding_completed FROM users WHERE id = ?1")
        .bind(&auth.user_id)
        .fetch_optional(&state.db_pool)
        .await;

    match user_query {
        Ok(Some(row)) => {
            let onboarding_completed: Option<i32> = row.get("onboarding_completed");
            if onboarding_completed.unwrap_or(0) == 1 {
                // User already completed onboarding, redirect to dashboard
                return (StatusCode::SEE_OTHER, [("Location", "/dashboard")]).into_response();
            }
            // Load existing partial onboarding data from database
            let user_data = sqlx::query(
                "SELECT dietary_restrictions, household_size, skill_level, weeknight_availability FROM users WHERE id = ?1"
            )
            .bind(&auth.user_id)
            .fetch_one(&state.db_pool)
            .await;

            let (dietary_restrictions, household_size_val, skill_level, weeknight_availability) =
                match user_data {
                    Ok(row) => {
                        let dietary_str: Option<String> = row.get("dietary_restrictions");
                        let dietary: Vec<String> = dietary_str
                            .and_then(|s| serde_json::from_str(&s).ok())
                            .unwrap_or_default();

                        let household: Option<i32> = row.get("household_size");
                        let household_str = household.map(|h| h.to_string()).unwrap_or_default();

                        let skill: Option<String> = row.get("skill_level");
                        let availability: Option<String> = row.get("weeknight_availability");

                        (
                            dietary,
                            household_str,
                            skill.unwrap_or_default(),
                            availability,
                        )
                    }
                    Err(_) => (Vec::new(), String::new(), String::new(), None),
                };

            // Parse availability JSON
            let (availability_start, availability_duration) = weeknight_availability
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .map(|v| {
                    let start = v["start"].as_str().unwrap_or("18:00").to_string();
                    let duration = v["duration_minutes"].as_u64().unwrap_or(45).to_string();
                    (start, duration)
                })
                .unwrap_or((String::from("18:00"), String::from("45")));

            // Determine which step to show (from query param or default to 1)
            let current_step = query.step.unwrap_or(1).clamp(1, 4);

            let template = OnboardingPageTemplate {
                error: String::new(),
                user: Some(()),
                current_step,
                dietary_restrictions,
                allergens: String::new(), // Allergens are merged into dietary_restrictions
                household_size: household_size_val,
                skill_level,
                availability_start,
                availability_duration,
            };
            Html(template.render().unwrap()).into_response()
        }
        Ok(None) => {
            // User not found, show error
            (StatusCode::NOT_FOUND, "User not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query user onboarding status: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}

/// POST /onboarding/step/1 - Save dietary restrictions and move to step 2
#[tracing::instrument(skip(state, auth, body))]
pub async fn post_onboarding_step_1(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    body: Bytes,
) -> Response {
    let body_str = String::from_utf8_lossy(&body);
    let form = OnboardingForm::from_form_data(&body_str);

    // Combine dietary restrictions and allergens
    let mut dietary_restrictions = form.dietary_restrictions;
    if !form.allergens.is_empty() {
        for allergen in form.allergens.split(',') {
            let trimmed = allergen.trim();
            if !trimmed.is_empty() {
                dietary_restrictions.push(trimmed.to_string());
            }
        }
    }

    // Emit event via command
    let command = user::SetDietaryRestrictionsCommand {
        user_id: auth.user_id,
        dietary_restrictions,
    };

    match user::set_dietary_restrictions(command, &state.evento_executor).await {
        Ok(_) => {
            // Wait for read model projection
            // Redirect to step 2
            (StatusCode::OK, [("ts-location", "/onboarding?step=2")]).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to set dietary restrictions: {:?}", e);
            (StatusCode::OK, [("ts-location", "/onboarding?step=1")]).into_response()
        }
    }
}

/// POST /onboarding/step/2 - Save household size and move to step 3
#[tracing::instrument(skip(state, auth, body))]
pub async fn post_onboarding_step_2(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    body: Bytes,
) -> Response {
    let body_str = String::from_utf8_lossy(&body);
    let form = OnboardingForm::from_form_data(&body_str);

    // Validate and parse household_size
    let household_size = if form.household_size.is_empty() {
        2
    } else {
        match form.household_size.parse::<u8>() {
            Ok(size) if (1..=10).contains(&size) => size,
            _ => {
                // Return to step 2 with error
                return (StatusCode::OK, [("ts-location", "/onboarding?step=2")]).into_response();
            }
        }
    };

    // Emit event via command
    let command = user::SetHouseholdSizeCommand {
        user_id: auth.user_id,
        household_size,
    };

    match user::set_household_size(command, &state.evento_executor).await {
        Ok(_) => (StatusCode::OK, [("ts-location", "/onboarding?step=3")]).into_response(),
        Err(e) => {
            tracing::error!("Failed to set household size: {:?}", e);
            (StatusCode::OK, [("ts-location", "/onboarding?step=2")]).into_response()
        }
    }
}

/// POST /onboarding/step/3 - Save skill level and move to step 4
#[tracing::instrument(skip(state, auth, body))]
pub async fn post_onboarding_step_3(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    body: Bytes,
) -> Response {
    let body_str = String::from_utf8_lossy(&body);
    let form = OnboardingForm::from_form_data(&body_str);

    let skill_level = if form.skill_level.is_empty() {
        "intermediate".to_string()
    } else {
        form.skill_level
    };

    // Emit event via command
    let command = user::SetSkillLevelCommand {
        user_id: auth.user_id,
        skill_level,
    };

    match user::set_skill_level(command, &state.evento_executor).await {
        Ok(_) => (StatusCode::OK, [("ts-location", "/onboarding?step=4")]).into_response(),
        Err(e) => {
            tracing::error!("Failed to set skill level: {:?}", e);
            (StatusCode::OK, [("ts-location", "/onboarding?step=3")]).into_response()
        }
    }
}

/// POST /onboarding/step/4 - Save availability and complete onboarding
#[tracing::instrument(skip(state, auth, body))]
pub async fn post_onboarding_step_4(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    body: Bytes,
) -> Response {
    let body_str = String::from_utf8_lossy(&body);
    let form = OnboardingForm::from_form_data(&body_str);

    let availability_start = if form.availability_start.is_empty() {
        "18:00".to_string()
    } else {
        form.availability_start
    };

    let availability_duration = form.availability_duration.parse::<u32>().unwrap_or(45);

    let weeknight_availability =
        json!({"start": availability_start, "duration_minutes": availability_duration}).to_string();

    // Emit availability event
    let availability_command = user::SetWeeknightAvailabilityCommand {
        user_id: auth.user_id.clone(),
        weeknight_availability,
    };

    match user::set_weeknight_availability(availability_command, &state.evento_executor).await {
        Ok(_) => {
            // Now mark profile as completed
            let complete_command = user::CompleteProfileCommand {
                user_id: auth.user_id,
            };

            match user::complete_profile(complete_command, &state.evento_executor).await {
                Ok(_) => {
                    // Redirect to dashboard
                    (StatusCode::OK, [("ts-location", "/dashboard")]).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to complete profile: {:?}", e);
                    (StatusCode::OK, [("ts-location", "/onboarding?step=4")]).into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to set weeknight availability: {:?}", e);
            (StatusCode::OK, [("ts-location", "/onboarding?step=4")]).into_response()
        }
    }
}

/// GET /onboarding/skip - Skip onboarding and apply defaults
///
/// AC #7: User can skip onboarding (optional) - defaults applied
#[tracing::instrument(skip(state, auth))]
pub async fn get_onboarding_skip(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
) -> Response {
    // Emit all step events with default values
    let _ = user::set_dietary_restrictions(
        user::SetDietaryRestrictionsCommand {
            user_id: auth.user_id.clone(),
            dietary_restrictions: Vec::new(),
        },
        &state.evento_executor,
    )
    .await;

    let _ = user::set_household_size(
        user::SetHouseholdSizeCommand {
            user_id: auth.user_id.clone(),
            household_size: 2,
        },
        &state.evento_executor,
    )
    .await;

    let _ = user::set_skill_level(
        user::SetSkillLevelCommand {
            user_id: auth.user_id.clone(),
            skill_level: "intermediate".to_string(),
        },
        &state.evento_executor,
    )
    .await;

    let _ = user::set_weeknight_availability(
        user::SetWeeknightAvailabilityCommand {
            user_id: auth.user_id.clone(),
            weeknight_availability: json!({"start": "18:00", "duration_minutes": 45}).to_string(),
        },
        &state.evento_executor,
    )
    .await;

    // Finally, mark profile as completed
    match user::complete_profile(
        user::CompleteProfileCommand {
            user_id: auth.user_id,
        },
        &state.evento_executor,
    )
    .await
    {
        Ok(()) => (StatusCode::SEE_OTHER, [("Location", "/dashboard")]).into_response(),
        Err(e) => {
            tracing::error!("Failed to skip onboarding: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to complete profile",
            )
                .into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "pages/profile.html")]
pub struct ProfilePageTemplate {
    pub error: String,
    pub success: bool,
    pub user: Option<()>,
    pub dietary_restrictions: Vec<String>,
    pub allergens: String,
    pub household_size: String,
    pub skill_level: String,
    pub availability_start: String,
    pub availability_duration: String,
    pub favorite_count: i64,
}

/// GET /profile - Display profile editing page
///
/// AC #1: Profile page displays current preferences in editable form
#[tracing::instrument(skip(state, auth))]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Query(query): Query<SuccessQuery>,
) -> Response {
    // Query user profile data from read model (including favorite_count)
    let user_data = sqlx::query(
        "SELECT dietary_restrictions, household_size, skill_level, weeknight_availability, favorite_count FROM users WHERE id = ?1"
    )
    .bind(&auth.user_id)
    .fetch_one(&state.db_pool)
    .await;

    match user_data {
        Ok(row) => {
            // Parse dietary_restrictions from JSON
            let dietary_str: Option<String> = row.get("dietary_restrictions");
            let dietary: Vec<String> = dietary_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            // Get household_size
            let household: Option<i32> = row.get("household_size");
            let household_str = household.map(|h| h.to_string()).unwrap_or_default();

            // Get skill_level
            let skill: Option<String> = row.get("skill_level");
            let skill_str = skill.unwrap_or_default();

            // Parse weeknight_availability JSON
            let availability: Option<String> = row.get("weeknight_availability");
            let (availability_start, availability_duration) = availability
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .map(|v| {
                    let start = v["start"].as_str().unwrap_or("18:00").to_string();
                    let duration = v["duration_minutes"].as_u64().unwrap_or(45).to_string();
                    (start, duration)
                })
                .unwrap_or((String::from("18:00"), String::from("45")));

            // Get favorite_count from users table (O(1) query via subscription)
            let favorite_count: i32 = row.get("favorite_count");

            let template = ProfilePageTemplate {
                error: String::new(),
                success: query.updated.unwrap_or(false),
                user: Some(()),
                dietary_restrictions: dietary,
                allergens: String::new(), // Allergens merged into dietary_restrictions
                household_size: household_str,
                skill_level: skill_str,
                availability_start,
                availability_duration,
                favorite_count: favorite_count as i64,
            };

            Html(template.render().unwrap()).into_response()
        }
        Err(sqlx::Error::RowNotFound) => {
            // User not found in read model (JWT valid but user deleted/corrupted)
            tracing::warn!("User not found in read model: user_id={}", auth.user_id);
            (StatusCode::SEE_OTHER, [("Location", "/login")]).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query user profile: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SuccessQuery {
    updated: Option<bool>,
}

/// POST /profile - Update user profile (creates ProfileUpdated event)
///
/// AC #2, #3, #4: User can modify profile, changes validated, successful save shows confirmation
#[tracing::instrument(skip(state, auth, body))]
pub async fn post_profile(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    body: Bytes,
) -> Response {
    let body_str = String::from_utf8_lossy(&body);
    let form = OnboardingForm::from_form_data(&body_str);

    // Clone form fields for error handling
    let form_allergens = form.allergens.clone();
    let form_household_size = form.household_size.clone();
    let form_skill_level = form.skill_level.clone();
    let form_availability_start = form.availability_start.clone();
    let form_availability_duration = form.availability_duration.clone();

    // Combine dietary restrictions and allergens
    let mut dietary_restrictions = form.dietary_restrictions.clone();
    if !form.allergens.is_empty() {
        for allergen in form.allergens.split(',') {
            let trimmed = allergen.trim();
            if !trimmed.is_empty() {
                dietary_restrictions.push(trimmed.to_string());
            }
        }
    }

    // Parse household_size (domain layer validates range 1-20)
    let household_size = if form.household_size.is_empty() {
        None
    } else {
        form.household_size.parse::<u8>().ok()
    };

    // Parse skill_level
    let skill_level = if form.skill_level.is_empty() {
        None
    } else {
        Some(form.skill_level)
    };

    // Parse weeknight_availability
    let weeknight_availability = if !form.availability_start.is_empty() {
        let availability_duration = form.availability_duration.parse::<u32>().unwrap_or(45);
        Some(
            json!({"start": form.availability_start, "duration_minutes": availability_duration})
                .to_string(),
        )
    } else {
        None
    };

    // Create UpdateProfileCommand
    let command = user::UpdateProfileCommand {
        user_id: auth.user_id.clone(),
        dietary_restrictions: Some(dietary_restrictions.clone()),
        household_size,
        skill_level,
        weeknight_availability,
    };

    // Invoke domain command
    match user::update_profile(command, &state.evento_executor).await {
        Ok(_) => {
            // Success - use optimistic UI rendering with form data to avoid projection lag
            // This ensures user immediately sees their changes without waiting for read model update
            // Query favorite count from users table (O(1) query)
            let favorite_count =
                sqlx::query_scalar::<_, i32>("SELECT favorite_count FROM users WHERE id = ?1")
                    .bind(&auth.user_id)
                    .fetch_one(&state.db_pool)
                    .await
                    .unwrap_or(0);

            let template = ProfilePageTemplate {
                error: String::new(),
                success: true,
                user: Some(()),
                dietary_restrictions,
                allergens: form_allergens,
                household_size: form_household_size,
                skill_level: form_skill_level,
                availability_start: form_availability_start,
                availability_duration: form_availability_duration,
                favorite_count: favorite_count as i64,
            };

            Html(template.render().unwrap()).into_response()
        }
        Err(user::UserError::ValidationError(msg)) => {
            // Query favorite count from users table (O(1) query)
            let favorite_count =
                sqlx::query_scalar::<_, i32>("SELECT favorite_count FROM users WHERE id = ?1")
                    .bind(&auth.user_id)
                    .fetch_one(&state.db_pool)
                    .await
                    .unwrap_or(0);

            // Validation error - re-render form with error
            let template = ProfilePageTemplate {
                error: msg,
                success: false,
                user: Some(()),
                dietary_restrictions,
                allergens: form_allergens,
                household_size: form_household_size,
                skill_level: form_skill_level,
                availability_start: form_availability_start,
                availability_duration: form_availability_duration,
                favorite_count: favorite_count as i64,
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Html(template.render().unwrap()),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update profile: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}

// ========================
// Subscription Management
// ========================

#[derive(Template)]
#[template(path = "pages/subscription.html")]
pub struct SubscriptionPageTemplate {
    pub error: String,
    pub user: Option<()>,
    pub tier: String,      // "free" or "premium"
    pub recipe_count: i32, // Current recipe count for free tier
    pub is_premium: bool,  // Derived: tier == "premium"
}

/// GET /subscription - Display subscription management page
///
/// AC #1, AC #2: Show current tier, premium benefits, pricing, upgrade button
#[tracing::instrument(skip(state, auth))]
pub async fn get_subscription(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
) -> Response {
    // Query user tier and recipe_count from read model
    let user_result = sqlx::query("SELECT tier, recipe_count FROM users WHERE id = ?1")
        .bind(&auth.user_id)
        .fetch_optional(&state.db_pool)
        .await;

    match user_result {
        Ok(Some(row)) => {
            let tier: String = row.get("tier");
            let recipe_count: i32 = row.get("recipe_count");
            let is_premium = tier == "premium";

            let template = SubscriptionPageTemplate {
                error: String::new(),
                user: Some(()),
                tier,
                recipe_count,
                is_premium,
            };

            Html(template.render().unwrap()).into_response()
        }
        Ok(None) => {
            tracing::error!("User not found: {}", auth.user_id);
            (StatusCode::INTERNAL_SERVER_ERROR, "User not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}

/// POST /subscription/upgrade - Create Stripe Checkout Session and redirect to Stripe
///
/// AC #3, AC #4: Redirect to Stripe Checkout hosted page for payment
#[tracing::instrument(skip(state, auth))]
pub async fn post_subscription_upgrade(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
) -> Response {
    // Query user email for Stripe Checkout
    let user_result = sqlx::query("SELECT email, tier FROM users WHERE id = ?1")
        .bind(&auth.user_id)
        .fetch_optional(&state.db_pool)
        .await;

    match user_result {
        Ok(Some(row)) => {
            let email: String = row.get("email");
            let tier: String = row.get("tier");

            // Prevent duplicate upgrade if already premium
            if tier == "premium" {
                tracing::warn!(
                    "User {} already premium, redirecting to subscription page",
                    auth.user_id
                );
                return axum::response::Redirect::to("/subscription").into_response();
            }

            // Initialize Stripe client
            let client = stripe::Client::new(&state.stripe_secret_key);

            // Create URL strings with proper lifetimes
            let success_url = format!("{}/subscription/success", state.base_url);
            let cancel_url = format!("{}/subscription", state.base_url);

            // Create Checkout Session
            let mut create_params = stripe::CreateCheckoutSession::new();
            create_params.mode = Some(stripe::CheckoutSessionMode::Subscription);
            create_params.customer_email = Some(&email);
            create_params.success_url = Some(&success_url);
            create_params.cancel_url = Some(&cancel_url);

            // Line items with Price ID
            create_params.line_items = Some(vec![stripe::CreateCheckoutSessionLineItems {
                price: Some(state.stripe_price_id.clone()),
                quantity: Some(1),
                ..Default::default()
            }]);

            // Metadata to identify user in webhook
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("user_id".to_string(), auth.user_id.clone());
            create_params.metadata = Some(metadata);

            // Create session
            match stripe::CheckoutSession::create(&client, create_params).await {
                Ok(session) => {
                    if let Some(url) = session.url {
                        tracing::info!(
                            "Created Stripe Checkout Session for user {}: {}",
                            auth.user_id,
                            session.id
                        );
                        axum::response::Redirect::to(&url).into_response()
                    } else {
                        tracing::error!("Stripe Checkout Session created but no URL returned");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to create checkout session",
                        )
                            .into_response()
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create Stripe Checkout Session: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to create checkout session",
                    )
                        .into_response()
                }
            }
        }
        Ok(None) => {
            tracing::error!("User not found: {}", auth.user_id);
            (StatusCode::INTERNAL_SERVER_ERROR, "User not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "pages/subscription-success.html")]
pub struct SubscriptionSuccessTemplate {
    pub user: Option<()>,
}

/// GET /subscription/success - Payment confirmation page
///
/// AC #7: User redirected back to /subscription/success after successful payment
#[tracing::instrument(skip(_auth))]
pub async fn get_subscription_success(Extension(_auth): Extension<Auth>) -> Response {
    let template = SubscriptionSuccessTemplate { user: Some(()) };

    Html(template.render().unwrap()).into_response()
}
