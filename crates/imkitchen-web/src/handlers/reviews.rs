use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use imkitchen_recipe::{
    commands::{
        CreateReviewCommand, DeleteReviewCommand, EditReviewCommand, RateRecipeCommand,
        VoteHelpfulnessCommand,
    },
    domain::rating::StarRating,
    projections::ReviewSummary,
};

// Templates for rendering review components
#[derive(Template)]
#[template(source = "<div>Rating stars component</div>", ext = "html")]
pub struct RatingStarsTemplate {
    pub mode: String,
    pub recipe_id: Option<Uuid>,
    pub current_rating: Option<u8>,
    pub average_rating: Option<f64>,
    pub total_ratings: Option<u32>,
    pub show_details: Option<bool>,
    pub compact: Option<bool>,
}

#[derive(Template)]
#[template(
    source = "<form><textarea>Review text</textarea><button>Submit Review</button></form>",
    ext = "html"
)]
pub struct ReviewFormTemplate {
    pub recipe_id: Option<Uuid>,
    pub rating_id: Option<Uuid>,
    pub review_id: Option<Uuid>,
    pub edit_mode: bool,
    pub existing_review_text: Option<String>,
    pub existing_photos: Option<Vec<String>>,
    pub validation_errors: Option<std::collections::HashMap<String, String>>,
}

#[derive(Template)]
#[template(source = "<div>Reviews ({{ total_count }})</div>", ext = "html")]
pub struct ReviewListTemplate {
    pub recipe_id: Uuid,
    pub reviews: Vec<ReviewSummary>,
    pub current_filter: String,
    pub sort_by: String,
    pub can_moderate: bool,
    pub total_count: u32,
    pub page: u32,
    pub has_more: bool,
}

#[derive(Template)]
#[template(
    source = "<div>Rating Distribution: {{ average_rating }} avg</div>",
    ext = "html"
)]
pub struct RatingDistributionTemplate {
    pub recipe_id: Uuid,
    pub average_rating: f64,
    pub total_ratings: u32,
    pub distribution: Vec<u32>,
    pub confidence_score: Option<f64>,
    pub most_common_rating: Option<u8>,
    pub recommendation_percentage: Option<f64>,
    pub positive_rating_percentage: f64,
    pub average_helpfulness_score: Option<f64>,
    pub verified_review_count: Option<u32>,
    pub reviews_with_photos: u32,
    pub current_filter: String,
    pub show_detailed_stats: bool,
}

#[derive(Template)]
#[template(
    source = "<div>Moderation Panel: {{ pending_count }} pending</div>",
    ext = "html"
)]
pub struct ReviewModerationPanelTemplate {
    pub reviews: Vec<ReviewSummary>,
    pub filter: String,
    pub sort_by: String,
    pub total_count: u32,
    pub pending_count: u32,
    pub flagged_count: u32,
    pub auto_approved_count: u32,
    pub page: u32,
    pub has_more: bool,
}

// Form data structures with validation
#[derive(Deserialize, Validate)]
pub struct RateRecipeForm {
    pub recipe_id: Uuid,
    #[validate(range(min = 1, max = 5))]
    pub star_rating: u8,
}

#[derive(Deserialize, Validate)]
pub struct CreateReviewForm {
    pub recipe_id: Uuid,
    pub rating_id: Option<Uuid>,
    #[validate(length(min = 10, max = 2000))]
    pub review_text: String,
    pub photos: Option<Vec<String>>,
}

#[derive(Deserialize, Validate)]
pub struct EditReviewForm {
    #[validate(length(min = 10, max = 2000))]
    pub review_text: String,
    pub photos: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ReviewQueryParams {
    pub rating: Option<u8>,
    pub sort: Option<String>,
    pub photos: Option<bool>,
    pub verified: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct HelpfulnessForm {
    pub helpful: bool,
}

#[derive(Deserialize)]
pub struct ModerationQueryParams {
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Deserialize, Validate)]
pub struct FlagReviewForm {
    #[validate(length(min = 10, max = 500))]
    pub flag_reason: String,
    pub flag_category: Option<String>,
}

/// Statistics for the moderation queue (temporary struct for this handler)
#[derive(Debug, Clone)]
pub struct ModerationQueueStats {
    pub total_reviews: u32,
    pub pending_count: u32,
    pub flagged_count: u32,
    pub approved_count: u32,
    pub rejected_count: u32,
    pub auto_approved_count: u32,
    pub manual_review_required: u32,
}

// Rating Handlers

/// POST /recipes/{id}/ratings - Submit a star rating
pub async fn rate_recipe(
    Path(recipe_id): Path<Uuid>,
    State(_state): State<AppState>,
    Form(form): Form<RateRecipeForm>,
) -> impl IntoResponse {
    // Validate form
    if let Err(validation_errors) = form.validate() {
        let error_msg = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| format!("{}: {}", field, errors[0]))
            .collect::<Vec<_>>()
            .join(", ");

        return Html(format!(
            r#"<div class="text-red-600 text-sm">Error: {}</div>"#,
            error_msg
        ));
    }

    // Create star rating
    let star_rating = match StarRating::new(form.star_rating) {
        Ok(rating) => rating,
        Err(_) => {
            return Html(
                r#"<div class="text-red-600 text-sm">Invalid rating value</div>"#.to_string(),
            );
        }
    };

    // Create rating command (in real implementation, this would use command handler)
    let _command = RateRecipeCommand {
        recipe_id: form.recipe_id,
        user_id: Uuid::new_v4(), // Would come from auth context
        star_rating,
    };

    // Return updated rating component
    let template = RatingStarsTemplate {
        mode: "display".to_string(),
        recipe_id: Some(recipe_id),
        current_rating: Some(form.star_rating),
        average_rating: Some(form.star_rating as f64),
        total_ratings: Some(1),
        show_details: Some(true),
        compact: Some(false),
    };

    Html(template.render().unwrap())
}

/// GET /recipes/{id}/reviews - List reviews for a recipe
pub async fn list_reviews(
    Path(recipe_id): Path<Uuid>,
    Query(params): Query<ReviewQueryParams>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Build query from parameters - in real implementation, this would use proper query handlers
    // For now, we'll just create mock data based on the parameters

    // Mock data for now - in real implementation, this would query the database
    let reviews = create_mock_review_list(recipe_id);
    let current_filter = if let Some(rating) = params.rating {
        rating.to_string()
    } else if params.photos.unwrap_or(false) {
        "photos".to_string()
    } else {
        "all".to_string()
    };

    let template = ReviewListTemplate {
        recipe_id,
        reviews,
        current_filter,
        sort_by: params.sort.unwrap_or_else(|| "newest".to_string()),
        can_moderate: false, // Would come from auth context
        total_count: 0,
        page: params.page.unwrap_or(1),
        has_more: false,
    };

    Html(template.render().unwrap())
}

/// POST /reviews - Create a new review
pub async fn create_review(
    State(_state): State<AppState>,
    Form(form): Form<CreateReviewForm>,
) -> impl IntoResponse {
    // Validate form
    if let Err(validation_errors) = form.validate() {
        let mut errors = std::collections::HashMap::new();
        for (field, field_errors) in validation_errors.field_errors() {
            errors.insert(field.to_string(), field_errors[0].to_string());
        }

        let template = ReviewFormTemplate {
            recipe_id: Some(form.recipe_id),
            rating_id: form.rating_id,
            review_id: None,
            edit_mode: false,
            existing_review_text: Some(form.review_text),
            existing_photos: form.photos,
            validation_errors: Some(errors),
        };

        return Html(template.render().unwrap());
    }

    // Create review command (in real implementation, this would use command handler)
    let _command = CreateReviewCommand {
        recipe_id: form.recipe_id,
        user_id: Uuid::new_v4(), // Would come from auth context
        rating_id: form.rating_id.unwrap_or_else(Uuid::new_v4),
        review_text: form.review_text,
        photos: form.photos.unwrap_or_default(),
    };

    // Return success message and updated review list
    let target_url = format!("/recipes/{}/reviews", form.recipe_id);
    let target_element = "#review-list";
    Html(format!(
        r#"<div ts-req="{}" ts-target="{}" class="text-green-600 text-sm mb-4">Review submitted successfully!</div>"#,
        target_url, target_element
    ))
}

/// PUT /reviews/{id}/edit - Edit an existing review
pub async fn edit_review(
    Path(review_id): Path<Uuid>,
    State(_state): State<AppState>,
    Form(form): Form<EditReviewForm>,
) -> impl IntoResponse {
    // Validate form
    if let Err(validation_errors) = form.validate() {
        let mut errors = std::collections::HashMap::new();
        for (field, field_errors) in validation_errors.field_errors() {
            errors.insert(field.to_string(), field_errors[0].to_string());
        }

        let template = ReviewFormTemplate {
            recipe_id: None,
            rating_id: None,
            review_id: Some(review_id),
            edit_mode: true,
            existing_review_text: Some(form.review_text),
            existing_photos: form.photos,
            validation_errors: Some(errors),
        };

        return Html(template.render().unwrap());
    }

    // Edit review command (in real implementation, this would use command handler)
    let _command = EditReviewCommand {
        review_id,
        user_id: Uuid::new_v4(), // Would come from auth context
        new_review_text: form.review_text,
    };

    // Return success message and updated review
    Html(format!(
        r#"<div ts-req="/reviews/{}/view" ts-target="[data-review-id='{}']" class="text-green-600 text-sm mb-2">Review updated successfully!</div>"#,
        review_id, review_id
    ))
}

/// DELETE /reviews/{id} - Delete a review
pub async fn delete_review(
    Path(review_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Delete review command (in real implementation, this would use command handler)
    let _command = DeleteReviewCommand {
        review_id,
        user_id: Uuid::new_v4(), // Would come from auth context
    };

    // Return empty content to remove the review from the list
    Html("".to_string())
}

/// POST /reviews/{id}/helpful - Mark review as helpful/not helpful
pub async fn update_review_helpfulness(
    Path(review_id): Path<Uuid>,
    State(_state): State<AppState>,
    Form(form): Form<HelpfulnessForm>,
) -> impl IntoResponse {
    // Update helpfulness command (in real implementation, this would use command handler)
    let _command = VoteHelpfulnessCommand {
        review_id,
        user_id: Uuid::new_v4(), // Would come from auth context
        is_helpful: form.helpful,
    };

    // Return updated helpfulness count (mock for now)
    let new_count = if form.helpful { 1 } else { 0 };
    Html(format!(
        r#"<span class="text-sm text-gray-600">{} people found this helpful</span>"#,
        new_count
    ))
}

/// DELETE /reviews/{id}/photos/{index} - Remove photo from review
pub async fn remove_review_photo(
    Path((_review_id, _photo_index)): Path<(Uuid, usize)>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // In real implementation, this would remove the photo from storage and update the review
    Html("".to_string())
}

/// GET /reviews/{id}/cancel-edit - Cancel review editing
pub async fn cancel_edit_review(
    Path(review_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Return the original review view (in real implementation, fetch from database)
    Html(format!(
        r#"<div class="text-gray-600 text-sm">Edit cancelled for review {}</div>"#,
        review_id
    ))
}

// Rating Distribution and Statistics

/// GET /recipes/{id}/rating-distribution - Get rating distribution for a recipe
pub async fn get_rating_distribution(
    Path(recipe_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Query rating aggregate (in real implementation, this would use query handler)

    // Mock data for now
    let template = RatingDistributionTemplate {
        recipe_id,
        average_rating: 4.2,
        total_ratings: 47,
        distribution: vec![2, 3, 8, 14, 20], // 1-star to 5-star counts
        confidence_score: Some(0.85),
        most_common_rating: Some(5),
        recommendation_percentage: Some(89.0),
        positive_rating_percentage: 72.3,
        average_helpfulness_score: Some(4.1),
        verified_review_count: Some(23),
        reviews_with_photos: 12,
        current_filter: "all".to_string(),
        show_detailed_stats: true,
    };

    Html(template.render().unwrap())
}

// Review Moderation (Admin only)

/// GET /admin/reviews/moderate - Admin review moderation panel
pub async fn review_moderation_panel(
    Query(params): Query<ModerationQueryParams>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Check admin permissions (in real implementation)

    // Build moderation query
    let filter = params.filter.unwrap_or_else(|| "pending".to_string());
    let sort_by = params.sort.unwrap_or_else(|| "newest".to_string());

    // Mock data for now
    let reviews = create_mock_moderation_reviews();

    let template = ReviewModerationPanelTemplate {
        reviews,
        filter,
        sort_by,
        total_count: 25,
        pending_count: 8,
        flagged_count: 3,
        auto_approved_count: 14,
        page: params.page.unwrap_or(1),
        has_more: false,
    };

    Html(template.render().unwrap())
}

/// POST /admin/reviews/{id}/approve - Approve a review
pub async fn approve_review(
    Path(review_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // In real implementation, this would use ModerateReviewCommand
    let _command = imkitchen_recipe::commands::ModerateReviewCommand {
        review_id,
        moderation_status: imkitchen_recipe::domain::rating::ReviewModerationStatus::Approved,
        moderation_reason: Some("Manually approved by moderator".to_string()),
        moderated_by: Uuid::new_v4(), // Would come from auth context
    };

    // Return updated moderation status
    Html(r#"<div class="inline-flex items-center px-2 py-1 text-xs font-medium bg-green-100 text-green-800 rounded-full">Approved</div>"#.to_string())
}

/// POST /admin/reviews/{id}/reject - Reject a review
pub async fn reject_review(
    Path(review_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // In real implementation, this would use ModerateReviewCommand
    let _command = imkitchen_recipe::commands::ModerateReviewCommand {
        review_id,
        moderation_status: imkitchen_recipe::domain::rating::ReviewModerationStatus::Rejected,
        moderation_reason: Some("Violated community guidelines".to_string()),
        moderated_by: Uuid::new_v4(), // Would come from auth context
    };

    // Return updated moderation status
    Html(r#"<div class="inline-flex items-center px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded-full">Rejected</div>"#.to_string())
}

/// POST /admin/reviews/bulk-approve - Bulk approve reviews
pub async fn bulk_approve_reviews(
    State(_state): State<AppState>,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Parse review IDs from form
    let review_ids: Vec<Uuid> = form
        .get("review_ids")
        .unwrap_or(&String::new())
        .split(',')
        .filter_map(|id| id.parse().ok())
        .collect();

    // In real implementation, this would use bulk moderation command handler
    let count = review_ids.len();

    // Return updated moderation panel section
    Html(format!(
        r#"<div class="bg-green-50 border border-green-200 rounded-md p-4 mb-4">
            <div class="flex">
                <div class="flex-shrink-0">
                    <svg class="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                    </svg>
                </div>
                <div class="ml-3">
                    <p class="text-sm font-medium text-green-800">Bulk approval completed</p>
                    <p class="text-sm text-green-700">Successfully approved {} reviews</p>
                </div>
            </div>
        </div>"#,
        count
    ))
}

/// POST /reviews/{id}/flag - Flag a review as inappropriate
pub async fn flag_review(
    Path(review_id): Path<Uuid>,
    State(_state): State<AppState>,
    Form(form): Form<FlagReviewForm>,
) -> impl IntoResponse {
    // Validate form
    if let Err(validation_errors) = form.validate() {
        let error_msg = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| format!("{}: {}", field, errors[0]))
            .collect::<Vec<_>>()
            .join(", ");

        return Html(format!(
            r#"<div class="text-red-600 text-sm">Error: {}</div>"#,
            error_msg
        ));
    }

    // In real implementation, this would use FlagReviewCommand
    let _command = imkitchen_recipe::commands::FlagReviewCommand::new(
        review_id,
        Uuid::new_v4(), // Would come from auth context
        form.flag_reason,
    );

    // Return success message
    Html(r#"<div class="text-orange-600 text-sm">Review flagged for moderation. Thank you for helping keep our community safe.</div>"#.to_string())
}

/// GET /admin/reviews/queue - Get moderation queue statistics
pub async fn get_moderation_queue_stats(State(_state): State<AppState>) -> impl IntoResponse {
    // In real implementation, this would query the database for current statistics
    let stats = ModerationQueueStats {
        total_reviews: 156,
        pending_count: 8,
        flagged_count: 3,
        approved_count: 140,
        rejected_count: 5,
        auto_approved_count: 125,
        manual_review_required: 11,
    };

    Html(format!(
        r#"<div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
            <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                <dt class="text-sm font-medium text-yellow-800">Pending</dt>
                <dd class="text-2xl font-bold text-yellow-900">{}</dd>
            </div>
            <div class="bg-red-50 border border-red-200 rounded-lg p-4">
                <dt class="text-sm font-medium text-red-800">Flagged</dt>
                <dd class="text-2xl font-bold text-red-900">{}</dd>
            </div>
            <div class="bg-green-50 border border-green-200 rounded-lg p-4">
                <dt class="text-sm font-medium text-green-800">Approved</dt>
                <dd class="text-2xl font-bold text-green-900">{}</dd>
            </div>
            <div class="bg-gray-50 border border-gray-200 rounded-lg p-4">
                <dt class="text-sm font-medium text-gray-800">Total</dt>
                <dd class="text-2xl font-bold text-gray-900">{}</dd>
            </div>
        </div>"#,
        stats.pending_count, stats.flagged_count, stats.approved_count, stats.total_reviews
    ))
}

// Helper functions for mock data

fn create_mock_review_list(_recipe_id: Uuid) -> Vec<ReviewSummary> {
    use chrono::Utc;
    use imkitchen_recipe::domain::rating::{ReviewModerationStatus, StarRating};

    vec![ReviewSummary {
        review_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        recipe_id: _recipe_id,
        rating_id: Uuid::new_v4(),
        star_rating: StarRating::new(5).unwrap(),
        review_text: "This recipe is absolutely amazing! I've made it three times already."
            .to_string(),
        review_preview: "This recipe is absolutely amazing! I've made it three times already."
            .to_string(),
        has_photos: true,
        photo_count: 1,
        helpfulness_score: 12,
        total_votes: 15,
        moderation_status: ReviewModerationStatus::Approved,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }]
}

fn create_mock_moderation_reviews() -> Vec<ReviewSummary> {
    use chrono::Utc;
    use imkitchen_recipe::domain::rating::{ReviewModerationStatus, StarRating};

    vec![ReviewSummary {
        review_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        recipe_id: Uuid::new_v4(),
        rating_id: Uuid::new_v4(),
        star_rating: StarRating::new(5).unwrap(),
        review_text: "These cookies turned out perfectly crispy on the outside and chewy inside!"
            .to_string(),
        review_preview:
            "These cookies turned out perfectly crispy on the outside and chewy inside!".to_string(),
        has_photos: false,
        photo_count: 0,
        helpfulness_score: 5,
        total_votes: 7,
        moderation_status: ReviewModerationStatus::Pending,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }]
}
