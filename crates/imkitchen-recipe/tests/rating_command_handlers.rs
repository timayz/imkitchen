use chrono::Utc;
use imkitchen_recipe::command_handlers::review_moderation::{
    ModerationQueueService, ReviewModerationCommandHandler,
};
use imkitchen_recipe::commands::{FlagReviewCommand, ModerateReviewCommand};
use imkitchen_recipe::domain::rating::{
    HelpfulnessVote, RecipeReview, ReviewModerationStatus, StarRating,
};
use imkitchen_recipe::domain::services::ReviewModerationService;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_review_moderation_command_handler_manual_moderation() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let handler = ReviewModerationCommandHandler::new(moderation_service);

    let review = create_test_review(ReviewModerationStatus::Pending);
    let moderator_id = Uuid::new_v4();

    let command = ModerateReviewCommand {
        review_id: review.review_id,
        moderation_status: ReviewModerationStatus::Approved,
        moderation_reason: Some("Manually reviewed and approved".to_string()),
        moderated_by: moderator_id,
    };

    let result = handler
        .handle_moderate_review(command, review.clone())
        .await;
    assert!(result.is_ok());

    let (updated_review, events) = result.unwrap();
    assert_eq!(
        updated_review.moderation_status,
        ReviewModerationStatus::Approved
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].review_id, review.review_id);
    assert_eq!(
        events[0].moderation_status,
        ReviewModerationStatus::Approved
    );
}

#[tokio::test]
async fn test_review_moderation_command_handler_auto_moderation() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let handler = ReviewModerationCommandHandler::new(moderation_service);

    // Test clean review (should be auto-approved)
    let clean_review = create_test_review_with_text(
        "This is an excellent recipe with clear instructions and great results!".to_string(),
    );

    let result = handler.handle_auto_moderate_review(&clean_review).await;
    assert!(result.is_ok());

    let (status, reason) = result.unwrap();
    assert_eq!(status, ReviewModerationStatus::Approved);
    assert!(reason.is_some());
    assert!(reason.unwrap().contains("Auto-approved"));

    // Test spam review (should be flagged)
    let spam_review =
        create_test_review_with_text("Click here to visit my site for amazing deals!".to_string());

    let result = handler.handle_auto_moderate_review(&spam_review).await;
    assert!(result.is_ok());

    let (status, reason) = result.unwrap();
    assert_eq!(status, ReviewModerationStatus::Flagged);
    assert!(reason.is_some());
    assert!(reason.unwrap().contains("spam"));
}

#[tokio::test]
async fn test_review_moderation_command_handler_flag_review() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let handler = ReviewModerationCommandHandler::new(moderation_service);

    let mut review = create_test_review(ReviewModerationStatus::Approved);
    let flagger_id = Uuid::new_v4();
    // Ensure flagger is different from review author
    assert_ne!(review.user_id, flagger_id);

    let command = FlagReviewCommand::new(
        review.review_id,
        flagger_id,
        "This review contains inappropriate content".to_string(),
    )
    .unwrap();

    let result = handler.handle_flag_review(command, review.clone()).await;
    assert!(result.is_ok());

    let (updated_review, events) = result.unwrap();
    assert_eq!(
        updated_review.moderation_status,
        ReviewModerationStatus::Flagged
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].review_id, review.review_id);
    assert_eq!(events[0].flagged_by, flagger_id);
}

#[tokio::test]
async fn test_review_moderation_command_handler_cannot_flag_own_review() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let handler = ReviewModerationCommandHandler::new(moderation_service);

    let review = create_test_review(ReviewModerationStatus::Approved);

    // Try to flag own review (should fail)
    let command = FlagReviewCommand::new(
        review.review_id,
        review.user_id, // Same user as review author
        "This is my own review".to_string(),
    )
    .unwrap();

    let result = handler.handle_flag_review(command, review).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot flag their own"));
}

#[tokio::test]
async fn test_review_moderation_command_handler_bulk_moderation() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let handler = ReviewModerationCommandHandler::new(moderation_service);

    let review1 = create_test_review(ReviewModerationStatus::Pending);
    let review2 = create_test_review(ReviewModerationStatus::Pending);
    let review3 = create_test_review(ReviewModerationStatus::Flagged);

    let reviews = vec![review1.clone(), review2.clone(), review3.clone()];
    let review_ids = vec![review1.review_id, review2.review_id];
    let moderator_id = Uuid::new_v4();

    let result = handler
        .handle_bulk_moderate_reviews(
            review_ids.clone(),
            ReviewModerationStatus::Approved,
            Some("Bulk approved after review".to_string()),
            moderator_id,
            reviews,
        )
        .await;

    assert!(result.is_ok());

    let (updated_reviews, events) = result.unwrap();
    assert_eq!(updated_reviews.len(), 2); // Only reviews in review_ids should be updated
    assert_eq!(events.len(), 2);

    for review in &updated_reviews {
        assert_eq!(review.moderation_status, ReviewModerationStatus::Approved);
        assert!(review_ids.contains(&review.review_id));
    }
}

#[tokio::test]
async fn test_review_moderation_command_handler_auto_approval_check() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let handler = ReviewModerationCommandHandler::new(moderation_service);

    // Test with clean review and good reputation
    let clean_review = create_test_review_with_text(
        "This is an excellent recipe with detailed instructions.".to_string(),
    );

    let should_approve = handler.should_auto_approve(&clean_review, Some(0.8)).await;
    assert!(should_approve);

    // Test with clean review but low reputation
    let should_not_approve = handler.should_auto_approve(&clean_review, Some(0.5)).await;
    assert!(!should_not_approve);

    // Test with problematic review
    let spam_review = create_test_review_with_text("Click here for free money!".to_string());

    let should_not_approve = handler.should_auto_approve(&spam_review, Some(0.9)).await;
    assert!(!should_not_approve);
}

#[tokio::test]
async fn test_moderation_queue_service_statistics() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let queue_service = ModerationQueueService::new(moderation_service);

    let reviews = vec![
        create_test_review(ReviewModerationStatus::Pending),
        create_test_review(ReviewModerationStatus::Pending),
        create_test_review(ReviewModerationStatus::Approved),
        create_test_review(ReviewModerationStatus::Approved),
        create_test_review(ReviewModerationStatus::Approved),
        create_test_review(ReviewModerationStatus::Flagged),
        create_test_review(ReviewModerationStatus::Rejected),
    ];

    let stats = queue_service.get_moderation_stats(&reviews).await;

    assert_eq!(stats.total_reviews, 7);
    assert_eq!(stats.pending_count, 2);
    assert_eq!(stats.approved_count, 3);
    assert_eq!(stats.flagged_count, 1);
    assert_eq!(stats.rejected_count, 1);
    assert_eq!(stats.manual_review_required, 3); // pending + flagged
}

#[tokio::test]
async fn test_moderation_queue_service_priority_reviews() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let queue_service = ModerationQueueService::new(moderation_service);

    // Create reviews with different statuses and timestamps
    let mut flagged_review = create_test_review(ReviewModerationStatus::Flagged);
    flagged_review.created_at = Utc::now() - chrono::Duration::hours(2);

    let mut pending_review = create_test_review_with_text(
        "This recipe might be controversial and needs manual review.".to_string(),
    );
    pending_review.moderation_status = ReviewModerationStatus::Pending;
    pending_review.created_at = Utc::now() - chrono::Duration::hours(1);

    let approved_review = create_test_review(ReviewModerationStatus::Approved);

    let reviews = vec![
        flagged_review.clone(),
        pending_review.clone(),
        approved_review,
    ];

    let priority_reviews = queue_service.get_priority_reviews(&reviews, 5).await;

    // Should include flagged review and pending review that requires manual review
    assert!(priority_reviews.len() >= 1); // At least the flagged review
    assert!(priority_reviews
        .iter()
        .any(|r| r.review_id == flagged_review.review_id));
}

#[tokio::test]
async fn test_moderation_queue_service_reviews_by_status() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let queue_service = ModerationQueueService::new(moderation_service);

    let pending_review1 = create_test_review(ReviewModerationStatus::Pending);
    let pending_review2 = create_test_review(ReviewModerationStatus::Pending);
    let approved_review = create_test_review(ReviewModerationStatus::Approved);
    let flagged_review = create_test_review(ReviewModerationStatus::Flagged);

    let reviews = vec![
        pending_review1.clone(),
        pending_review2.clone(),
        approved_review,
        flagged_review,
    ];

    // Test getting pending reviews
    let pending_reviews = queue_service
        .get_reviews_by_status(&reviews, ReviewModerationStatus::Pending, 1, 10)
        .await;

    assert_eq!(pending_reviews.len(), 2);
    assert!(pending_reviews
        .iter()
        .any(|r| r.review_id == pending_review1.review_id));
    assert!(pending_reviews
        .iter()
        .any(|r| r.review_id == pending_review2.review_id));

    // Test pagination
    let first_page = queue_service
        .get_reviews_by_status(&reviews, ReviewModerationStatus::Pending, 1, 1)
        .await;
    assert_eq!(first_page.len(), 1);

    let second_page = queue_service
        .get_reviews_by_status(&reviews, ReviewModerationStatus::Pending, 2, 1)
        .await;
    assert_eq!(second_page.len(), 1);

    // Ensure different reviews on different pages
    assert_ne!(first_page[0].review_id, second_page[0].review_id);
}

#[tokio::test]
async fn test_command_validation_and_error_handling() {
    let moderation_service = Arc::new(ReviewModerationService::new());
    let handler = ReviewModerationCommandHandler::new(moderation_service);

    let review = create_test_review(ReviewModerationStatus::Pending);

    // Test invalid moderation command (this would be caught at command level, but test handler behavior)
    let command = ModerateReviewCommand {
        review_id: review.review_id,
        moderation_status: ReviewModerationStatus::Rejected,
        moderation_reason: Some("Content violates guidelines".to_string()),
        moderated_by: Uuid::new_v4(),
    };

    let result = handler.handle_moderate_review(command, review).await;
    assert!(result.is_ok()); // Should succeed with valid data

    // Test flag command validation
    let flag_result = FlagReviewCommand::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "Too short".to_string(), // Less than 10 characters
    );
    assert!(flag_result.is_err()); // Should fail validation

    let valid_flag_result = FlagReviewCommand::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "This review contains inappropriate language and should be reviewed.".to_string(),
    );
    assert!(valid_flag_result.is_ok()); // Should pass validation
}

// Helper functions for creating test data
fn create_test_review(moderation_status: ReviewModerationStatus) -> RecipeReview {
    create_test_review_with_text_and_status(
        "This is a test review with sufficient length for validation.".to_string(),
        moderation_status,
    )
}

fn create_test_review_with_text(review_text: String) -> RecipeReview {
    create_test_review_with_text_and_status(review_text, ReviewModerationStatus::Pending)
}

fn create_test_review_with_text_and_status(
    review_text: String,
    moderation_status: ReviewModerationStatus,
) -> RecipeReview {
    let mut review = RecipeReview {
        review_id: Uuid::new_v4(),
        rating_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        recipe_id: Uuid::new_v4(),
        review_text,
        photos: vec![],
        helpfulness_score: 0,
        helpfulness_votes: vec![],
        moderation_status,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    review
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_moderation_workflow() {
        let moderation_service = Arc::new(ReviewModerationService::new());
        let handler = ReviewModerationCommandHandler::new(moderation_service.clone());
        let queue_service = ModerationQueueService::new(moderation_service);

        // Step 1: Create a review that gets auto-moderated
        let review = create_test_review_with_text(
            "This is an excellent recipe with very detailed instructions!".to_string(),
        );

        let (auto_status, _) = handler.handle_auto_moderate_review(&review).await.unwrap();
        assert_eq!(auto_status, ReviewModerationStatus::Approved);

        // Step 2: User flags the approved review
        let flagger_id = Uuid::new_v4();
        let flag_command = FlagReviewCommand::new(
            review.review_id,
            flagger_id,
            "This review seems fake and should be investigated.".to_string(),
        )
        .unwrap();

        let mut approved_review = review.clone();
        approved_review.moderation_status = ReviewModerationStatus::Approved;

        let (flagged_review, flag_events) = handler
            .handle_flag_review(flag_command, approved_review)
            .await
            .unwrap();

        assert_eq!(
            flagged_review.moderation_status,
            ReviewModerationStatus::Flagged
        );
        assert_eq!(flag_events.len(), 1);

        // Step 3: Admin manually reviews and approves
        let moderator_id = Uuid::new_v4();
        let moderate_command = ModerateReviewCommand {
            review_id: flagged_review.review_id,
            moderation_status: ReviewModerationStatus::Approved,
            moderation_reason: Some("Reviewed manually - content is appropriate".to_string()),
            moderated_by: moderator_id,
        };

        let (final_review, moderate_events) = handler
            .handle_moderate_review(moderate_command, flagged_review.clone())
            .await
            .unwrap();

        assert_eq!(
            final_review.moderation_status,
            ReviewModerationStatus::Approved
        );
        assert_eq!(moderate_events.len(), 1);

        // Step 4: Check queue statistics
        let reviews = vec![final_review];
        let stats = queue_service.get_moderation_stats(&reviews).await;
        assert_eq!(stats.approved_count, 1);
        assert_eq!(stats.flagged_count, 0);
        assert_eq!(stats.total_reviews, 1);
    }

    #[tokio::test]
    async fn test_bulk_moderation_workflow() {
        let moderation_service = Arc::new(ReviewModerationService::new());
        let handler = ReviewModerationCommandHandler::new(moderation_service);

        // Create multiple pending reviews
        let reviews = vec![
            create_test_review(ReviewModerationStatus::Pending),
            create_test_review(ReviewModerationStatus::Pending),
            create_test_review(ReviewModerationStatus::Flagged),
            create_test_review(ReviewModerationStatus::Pending),
        ];

        let review_ids = vec![
            reviews[0].review_id,
            reviews[1].review_id,
            reviews[3].review_id,
        ];
        let moderator_id = Uuid::new_v4();

        // Bulk approve selected reviews
        let (updated_reviews, events) = handler
            .handle_bulk_moderate_reviews(
                review_ids.clone(),
                ReviewModerationStatus::Approved,
                Some("Bulk approved after manual review".to_string()),
                moderator_id,
                reviews,
            )
            .await
            .unwrap();

        assert_eq!(updated_reviews.len(), 3);
        assert_eq!(events.len(), 3);

        for review in &updated_reviews {
            assert_eq!(review.moderation_status, ReviewModerationStatus::Approved);
            assert!(review_ids.contains(&review.review_id));
        }

        for event in &events {
            assert_eq!(event.moderation_status, ReviewModerationStatus::Approved);
            assert!(review_ids.contains(&event.review_id));
        }
    }
}
