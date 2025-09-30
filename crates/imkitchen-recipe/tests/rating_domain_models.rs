use chrono::Utc;
use imkitchen_recipe::domain::rating::{
    HelpfulnessVote, RecipeRating, RecipeReview, ReviewModerationStatus, StarRating,
};
use uuid::Uuid;

#[test]
fn test_star_rating_creation() {
    // Test valid star ratings
    for i in 1..=5 {
        let rating = StarRating::new(i);
        assert!(rating.is_ok(), "Rating {} should be valid", i);
        assert_eq!(rating.unwrap().value, i);
    }

    // Test invalid star ratings
    for i in [0, 6, 10, 255] {
        let rating = StarRating::new(i);
        assert!(rating.is_err(), "Rating {} should be invalid", i);
    }
}

#[test]
fn test_recipe_rating_creation() {
    let _rating_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let star_rating = StarRating::new(4).unwrap();

    let rating = RecipeRating::new(user_id, recipe_id, star_rating).unwrap();

    assert_eq!(rating.recipe_id, recipe_id);
    assert_eq!(rating.user_id, user_id);
    assert_eq!(rating.star_rating.value, 4);
    assert!(rating.created_at <= Utc::now());
}

#[test]
fn test_recipe_review_creation_valid() {
    let rating_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let review_text =
        "This is a great recipe with excellent instructions and ingredients.".to_string();
    let photos = vec!["photo1.jpg".to_string(), "photo2.jpg".to_string()];

    let result = RecipeReview::new(
        rating_id,
        user_id,
        recipe_id,
        review_text.clone(),
        photos.clone(),
    );

    assert!(result.is_ok());
    let review = result.unwrap();
    assert_eq!(review.rating_id, rating_id);
    assert_eq!(review.user_id, user_id);
    assert_eq!(review.recipe_id, recipe_id);
    assert_eq!(review.review_text, review_text);
    assert_eq!(review.photos, photos);
    assert_eq!(review.helpfulness_score, 0);
    assert!(review.helpfulness_votes.is_empty());
    assert_eq!(review.moderation_status, ReviewModerationStatus::Pending);
    assert!(review.created_at <= Utc::now());
    assert!(review.updated_at <= Utc::now());
}

#[test]
fn test_recipe_review_creation_invalid() {
    let rating_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let photos = vec![];

    // Test review too short (less than 10 characters)
    let short_review = "Too short".to_string();
    let result = RecipeReview::new(rating_id, user_id, recipe_id, short_review, photos.clone());
    assert!(result.is_err());

    // Test review too long (more than 2000 characters)
    let long_review = "a".repeat(2001);
    let result = RecipeReview::new(rating_id, user_id, recipe_id, long_review, photos);
    assert!(result.is_err());

    // Test empty review
    let empty_review = "".to_string();
    let result = RecipeReview::new(rating_id, user_id, recipe_id, empty_review, vec![]);
    assert!(result.is_err());

    // Test whitespace-only review
    let whitespace_review = "   \n\t   ".to_string();
    let result = RecipeReview::new(rating_id, user_id, recipe_id, whitespace_review, vec![]);
    assert!(result.is_err());
}

#[test]
fn test_recipe_review_helpfulness_voting() {
    let rating_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let review_text = "This is a helpful review with detailed feedback.".to_string();

    let mut review = RecipeReview::new(rating_id, user_id, recipe_id, review_text, vec![]).unwrap();

    // Test adding helpful vote
    let helpful_vote = HelpfulnessVote::new(Uuid::new_v4(), true);
    let result = review.add_helpfulness_vote(helpful_vote.clone());
    assert!(result.is_ok());
    assert_eq!(review.helpfulness_score, 1);
    assert_eq!(review.helpfulness_votes.len(), 1);

    // Test adding unhelpful vote from different user
    let unhelpful_vote = HelpfulnessVote::new(Uuid::new_v4(), false);
    let result = review.add_helpfulness_vote(unhelpful_vote);
    assert!(result.is_ok());
    assert_eq!(review.helpfulness_score, 0); // 1 helpful + 1 unhelpful = 0
    assert_eq!(review.helpfulness_votes.len(), 2);

    // Test duplicate vote from same user
    let duplicate_vote = HelpfulnessVote::new(helpful_vote.user_id, false);
    let result = review.add_helpfulness_vote(duplicate_vote);
    assert!(result.is_err());
    assert_eq!(review.helpfulness_score, 0); // Should remain unchanged
    assert_eq!(review.helpfulness_votes.len(), 2); // Should remain unchanged
}

#[test]
fn test_helpfulness_vote_creation() {
    let user_id = Uuid::new_v4();

    // Test helpful vote
    let helpful_vote = HelpfulnessVote::new(user_id, true);
    assert_eq!(helpful_vote.user_id, user_id);
    assert!(helpful_vote.is_helpful);
    assert!(helpful_vote.voted_at <= Utc::now());

    // Test unhelpful vote
    let unhelpful_vote = HelpfulnessVote::new(user_id, false);
    assert_eq!(unhelpful_vote.user_id, user_id);
    assert!(!unhelpful_vote.is_helpful);
    assert!(unhelpful_vote.voted_at <= Utc::now());
}

#[test]
fn test_review_moderation_status_values() {
    // Test all moderation status values
    let statuses = vec![
        ReviewModerationStatus::Pending,
        ReviewModerationStatus::Approved,
        ReviewModerationStatus::Rejected,
        ReviewModerationStatus::Flagged,
    ];

    for status in statuses {
        // Test that all status values are valid
        let rating_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let recipe_id = Uuid::new_v4();
        let review_text = "This is a test review for moderation status testing.".to_string();

        let mut review =
            RecipeReview::new(rating_id, user_id, recipe_id, review_text, vec![]).unwrap();
        review.moderation_status = status;

        // Should be able to set any valid status
        assert_eq!(review.moderation_status, status);
    }
}

#[test]
fn test_review_update_operations() {
    let rating_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let original_text = "This is the original review text with sufficient length.".to_string();

    let mut review =
        RecipeReview::new(rating_id, user_id, recipe_id, original_text, vec![]).unwrap();
    let original_created_at = review.created_at;
    let original_updated_at = review.updated_at;

    // Simulate time passage
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Test updating review text
    let new_text =
        "This is the updated review text with much more detailed information.".to_string();
    review.review_text = new_text.clone();
    review.updated_at = Utc::now();

    assert_eq!(review.review_text, new_text);
    assert_eq!(review.created_at, original_created_at); // Should not change
    assert!(review.updated_at > original_updated_at); // Should be updated

    // Test updating moderation status
    review.moderation_status = ReviewModerationStatus::Approved;
    assert_eq!(review.moderation_status, ReviewModerationStatus::Approved);

    // Test adding photos
    let photos = vec![
        "updated_photo1.jpg".to_string(),
        "updated_photo2.jpg".to_string(),
    ];
    review.photos = photos.clone();
    assert_eq!(review.photos, photos);
}

#[test]
fn test_complex_helpfulness_scoring() {
    let rating_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let review_text = "This is a comprehensive review for helpfulness testing.".to_string();

    let mut review = RecipeReview::new(rating_id, user_id, recipe_id, review_text, vec![]).unwrap();

    // Add multiple helpful votes
    for i in 0..5 {
        let vote = HelpfulnessVote::new(Uuid::new_v4(), true);
        let result = review.add_helpfulness_vote(vote);
        assert!(result.is_ok());
        assert_eq!(review.helpfulness_score, i + 1);
    }

    // Add some unhelpful votes
    for i in 0..2 {
        let vote = HelpfulnessVote::new(Uuid::new_v4(), false);
        let result = review.add_helpfulness_vote(vote);
        assert!(result.is_ok());
        assert_eq!(review.helpfulness_score, 5 - (i + 1)); // 5 helpful - (i+1) unhelpful
    }

    // Final score should be 3 (5 helpful - 2 unhelpful)
    assert_eq!(review.helpfulness_score, 3);
    assert_eq!(review.helpfulness_votes.len(), 7);
}

#[test]
fn test_edge_cases() {
    let rating_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();

    // Test review with exactly minimum length (10 characters)
    let min_review = "1234567890".to_string(); // Exactly 10 characters
    let result = RecipeReview::new(rating_id, user_id, recipe_id, min_review, vec![]);
    assert!(result.is_ok());

    // Test review with exactly maximum length (2000 characters)
    let max_review = "a".repeat(2000);
    let result = RecipeReview::new(rating_id, user_id, recipe_id, max_review, vec![]);
    assert!(result.is_ok());

    // Test review with exactly minimum length - 1 (9 characters)
    let too_short = "123456789".to_string(); // 9 characters
    let result = RecipeReview::new(rating_id, user_id, recipe_id, too_short, vec![]);
    assert!(result.is_err());

    // Test review with exactly maximum length + 1 (2001 characters)
    let too_long = "a".repeat(2001);
    let result = RecipeReview::new(rating_id, user_id, recipe_id, too_long, vec![]);
    assert!(result.is_err());
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_rating_and_review_workflow() {
        // Create a rating
        let recipe_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let star_rating = StarRating::new(5).unwrap();
        let rating = RecipeRating::new(user_id, recipe_id, star_rating).unwrap();

        // Create a review for the rating
        let review_text = "Excellent recipe! Easy to follow and delicious results.".to_string();
        let photos = vec!["finished_dish.jpg".to_string()];
        let mut review =
            RecipeReview::new(rating.rating_id, user_id, recipe_id, review_text, photos).unwrap();

        // Verify initial state
        assert_eq!(review.moderation_status, ReviewModerationStatus::Pending);
        assert_eq!(review.helpfulness_score, 0);

        // Add helpfulness votes from community
        let voter1 = Uuid::new_v4();
        let voter2 = Uuid::new_v4();
        let voter3 = Uuid::new_v4();

        review
            .add_helpfulness_vote(HelpfulnessVote::new(voter1, true))
            .unwrap();
        review
            .add_helpfulness_vote(HelpfulnessVote::new(voter2, true))
            .unwrap();
        review
            .add_helpfulness_vote(HelpfulnessVote::new(voter3, false))
            .unwrap();

        // Verify helpfulness scoring
        assert_eq!(review.helpfulness_score, 1); // 2 helpful - 1 unhelpful
        assert_eq!(review.helpfulness_votes.len(), 3);

        // Moderate the review
        review.moderation_status = ReviewModerationStatus::Approved;

        // Verify final state
        assert_eq!(review.moderation_status, ReviewModerationStatus::Approved);
        assert_eq!(review.rating_id, rating.rating_id);
        assert_eq!(review.recipe_id, recipe_id);
        assert_eq!(rating.recipe_id, recipe_id);
        assert_eq!(rating.star_rating.value, 5);
    }
}
