use chrono::Utc;
use imkitchen_recipe::domain::rating::{
    HelpfulnessVote, RecipeRating, RecipeReview, ReviewModerationStatus, StarRating,
};
use imkitchen_recipe::domain::services::{
    RatingAggregationService, ReviewModerationService, StatisticalWeightingService,
};
use uuid::Uuid;

#[test]
fn test_rating_aggregation_service_weighted_average() {
    let service = RatingAggregationService::new();
    let recipe_id = Uuid::new_v4();

    // Test empty ratings
    let empty_ratings: Vec<RecipeRating> = vec![];
    let avg = service.calculate_weighted_average(&empty_ratings);
    assert_eq!(avg, 0.0);

    // Test single rating
    let single_rating = vec![create_test_rating(recipe_id, 4)];
    let avg = service.calculate_weighted_average(&single_rating);
    // Should be pulled toward global average (3.0) due to low sample size
    assert!(avg > 3.0 && avg < 4.0);

    // Test multiple ratings with same value
    let same_ratings = vec![
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 5),
    ];
    let avg = service.calculate_weighted_average(&same_ratings);
    // Should be closer to 5.0 with more samples but still pulled toward global average
    assert!(avg > 3.5 && avg < 5.0);

    // Test mixed ratings
    let mixed_ratings = vec![
        create_test_rating(recipe_id, 1),
        create_test_rating(recipe_id, 2),
        create_test_rating(recipe_id, 3),
        create_test_rating(recipe_id, 4),
        create_test_rating(recipe_id, 5),
    ];
    let avg = service.calculate_weighted_average(&mixed_ratings);
    // Average should be close to 3.0 (the global average)
    assert!(avg >= 2.8 && avg <= 3.2);
}

#[test]
fn test_rating_aggregation_service_distribution() {
    let service = RatingAggregationService::new();
    let recipe_id = Uuid::new_v4();

    // Test empty ratings
    let empty_ratings: Vec<RecipeRating> = vec![];
    let distribution = service.calculate_rating_distribution(&empty_ratings);
    assert_eq!(distribution, [0, 0, 0, 0, 0]);

    // Test various ratings
    let ratings = vec![
        create_test_rating(recipe_id, 1),
        create_test_rating(recipe_id, 1),
        create_test_rating(recipe_id, 2),
        create_test_rating(recipe_id, 3),
        create_test_rating(recipe_id, 3),
        create_test_rating(recipe_id, 3),
        create_test_rating(recipe_id, 4),
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 5),
    ];
    let distribution = service.calculate_rating_distribution(&ratings);
    // [1-star: 2, 2-star: 1, 3-star: 3, 4-star: 1, 5-star: 5]
    assert_eq!(distribution, [2, 1, 3, 1, 5]);
}

#[test]
fn test_rating_aggregation_service_confidence_score() {
    let service = RatingAggregationService::new();

    // Test confidence scores for different sample sizes
    assert_eq!(service.calculate_confidence_score(0), 0.0);
    assert_eq!(service.calculate_confidence_score(1), 0.3);
    assert_eq!(service.calculate_confidence_score(5), 0.3);
    assert_eq!(service.calculate_confidence_score(6), 0.6);
    assert_eq!(service.calculate_confidence_score(15), 0.6);
    assert_eq!(service.calculate_confidence_score(16), 0.8);
    assert_eq!(service.calculate_confidence_score(50), 0.8);
    assert_eq!(service.calculate_confidence_score(51), 0.95);
    assert_eq!(service.calculate_confidence_score(100), 0.95);
}

#[test]
fn test_review_moderation_service_content_filtering() {
    let service = ReviewModerationService::new();
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let rating_id = Uuid::new_v4();

    // Test clean review (should be approved)
    let clean_review = create_test_review(
        rating_id,
        user_id,
        recipe_id,
        "This is a wonderful recipe with clear instructions and great results!".to_string(),
    );
    let status = service.moderate_review(&clean_review);
    assert_eq!(status, ReviewModerationStatus::Approved);

    // Test spam content (should be flagged)
    let spam_review = create_test_review(
        rating_id,
        user_id,
        recipe_id,
        "Great recipe! Click here to visit my site for more recipes!".to_string(),
    );
    let status = service.moderate_review(&spam_review);
    assert_eq!(status, ReviewModerationStatus::Flagged);

    // Test low quality content (should be pending)
    let low_quality_review = create_test_review(
        rating_id,
        user_id,
        recipe_id,
        "ok".to_string(), // Too short, low quality
    );
    let status = service.moderate_review(&low_quality_review);
    assert_eq!(status, ReviewModerationStatus::Pending);

    // Test repetitive content (should be pending)
    let repetitive_review = create_test_review(
        rating_id,
        user_id,
        recipe_id,
        "aaaaaaaaaaaa this recipe is great".to_string(), // Repetitive characters
    );
    let status = service.moderate_review(&repetitive_review);
    assert_eq!(status, ReviewModerationStatus::Pending);
}

#[test]
fn test_review_moderation_service_manual_review_detection() {
    let service = ReviewModerationService::new();
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let rating_id = Uuid::new_v4();

    // Test review that doesn't require manual review
    let simple_review = create_test_review(
        rating_id,
        user_id,
        recipe_id,
        "This recipe turned out perfectly! Easy to follow and delicious.".to_string(),
    );
    assert!(!service.requires_manual_review(&simple_review));

    // Test review that requires manual review (contains uncertainty)
    let uncertain_review = create_test_review(
        rating_id,
        user_id,
        recipe_id,
        "This recipe is maybe good, not sure if I would recommend it.".to_string(),
    );
    assert!(service.requires_manual_review(&uncertain_review));

    // Test review with controversial content
    let controversial_review = create_test_review(
        rating_id,
        user_id,
        recipe_id,
        "This controversial recipe might be offensive to some people.".to_string(),
    );
    assert!(service.requires_manual_review(&controversial_review));
}

#[test]
fn test_statistical_weighting_service_helpfulness() {
    let service = StatisticalWeightingService::new();
    let user_id = Uuid::new_v4();

    // Test basic helpfulness weighting
    let base_score = 10;
    let weighted_score = service.calculate_helpfulness_weight(user_id, base_score);
    assert_eq!(weighted_score, 10.0); // Simple 1.0 weight factor for now
}

#[test]
fn test_statistical_weighting_service_time_decay() {
    let service = StatisticalWeightingService::new();

    // Test recent review (should have high weight)
    let recent_decay = service.calculate_time_decay_factor(0);
    assert_eq!(recent_decay, 1.0);

    // Test review from 1 day ago
    let one_day_decay = service.calculate_time_decay_factor(1);
    assert_eq!(one_day_decay, 0.99);

    // Test review from 100 days ago
    let old_decay = service.calculate_time_decay_factor(100);
    assert_eq!(old_decay, 0.1); // Should hit minimum due to decay

    // Test very old review (should have minimum weight)
    let very_old_decay = service.calculate_time_decay_factor(500);
    assert_eq!(very_old_decay, 0.1); // Minimum weight
}

#[test]
fn test_statistical_weighting_service_quality_score() {
    let service = StatisticalWeightingService::new();
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let rating_id = Uuid::new_v4();

    // Test short review without photos (lower quality)
    let short_review =
        create_test_review(rating_id, user_id, recipe_id, "Good recipe.".to_string());
    let quality_score = service.calculate_review_quality_score(&short_review);
    assert!(quality_score < 0.8); // Should be lower quality

    // Test detailed review with photos (higher quality)
    let mut detailed_review = create_test_review(
        rating_id,
        user_id,
        recipe_id,
        "This is an excellent recipe with very detailed instructions. I followed it exactly and the results were fantastic. The flavors were perfectly balanced and my family loved it. I will definitely make this again and would highly recommend it to anyone looking for a delicious and easy-to-follow recipe.".to_string(),
    );
    detailed_review.photos = vec!["photo1.jpg".to_string(), "photo2.jpg".to_string()];

    // Add some helpfulness votes
    detailed_review
        .add_helpfulness_vote(HelpfulnessVote::new(Uuid::new_v4(), true))
        .unwrap();
    detailed_review
        .add_helpfulness_vote(HelpfulnessVote::new(Uuid::new_v4(), true))
        .unwrap();

    let quality_score = service.calculate_review_quality_score(&detailed_review);
    assert!(quality_score > 0.7); // Should be higher quality
}

// Integration tests for combined service functionality
#[test]
fn test_rating_services_integration() {
    let aggregation_service = RatingAggregationService::new();
    let moderation_service = ReviewModerationService::new();
    let weighting_service = StatisticalWeightingService::new();

    let recipe_id = Uuid::new_v4();

    // Create a set of ratings and reviews
    let ratings = vec![
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 4),
        create_test_rating(recipe_id, 5),
        create_test_rating(recipe_id, 3),
        create_test_rating(recipe_id, 4),
    ];

    let review = create_test_review(
        ratings[0].rating_id,
        ratings[0].user_id,
        recipe_id,
        "Excellent recipe with clear instructions and amazing results!".to_string(),
    );

    // Test aggregation
    let weighted_avg = aggregation_service.calculate_weighted_average(&ratings);
    let distribution = aggregation_service.calculate_rating_distribution(&ratings);
    let confidence = aggregation_service.calculate_confidence_score(ratings.len() as u32);

    assert!(weighted_avg > 3.0 && weighted_avg < 5.0);
    assert_eq!(distribution, [0, 0, 1, 2, 2]); // [1★:0, 2★:0, 3★:1, 4★:2, 5★:2]
    assert_eq!(confidence, 0.3); // 5 ratings should give 0.3 confidence

    // Test moderation
    let moderation_status = moderation_service.moderate_review(&review);
    assert_eq!(moderation_status, ReviewModerationStatus::Approved);

    // Test quality scoring
    let quality_score = weighting_service.calculate_review_quality_score(&review);
    assert!(quality_score > 0.3); // Should be decent quality for basic review
}

#[test]
fn test_edge_cases_and_error_conditions() {
    let aggregation_service = RatingAggregationService::new();
    let moderation_service = ReviewModerationService::new();

    // Test with no ratings
    let empty_ratings: Vec<RecipeRating> = vec![];
    assert_eq!(
        aggregation_service.calculate_weighted_average(&empty_ratings),
        0.0
    );
    assert_eq!(
        aggregation_service.calculate_rating_distribution(&empty_ratings),
        [0, 0, 0, 0, 0]
    );

    // Test moderation with empty review text
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let rating_id = Uuid::new_v4();

    // This should be caught at domain model level, but test service behavior
    let empty_review = create_test_review(rating_id, user_id, recipe_id, "".to_string());
    let status = moderation_service.moderate_review(&empty_review);
    assert_eq!(status, ReviewModerationStatus::Pending); // Should require manual review
}

// Helper functions for creating test data
fn create_test_rating(recipe_id: Uuid, star_value: u8) -> RecipeRating {
    let user_id = Uuid::new_v4();
    let star_rating = StarRating::new(star_value).unwrap();
    RecipeRating::new(user_id, recipe_id, star_rating).unwrap()
}

fn create_test_review(
    rating_id: Uuid,
    user_id: Uuid,
    recipe_id: Uuid,
    review_text: String,
) -> RecipeReview {
    // For reviews that might not pass validation, create manually
    RecipeReview {
        review_id: Uuid::new_v4(),
        rating_id,
        user_id,
        recipe_id,
        review_text,
        photos: vec![],
        helpfulness_score: 0,
        helpfulness_votes: vec![],
        moderation_status: ReviewModerationStatus::Pending,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
