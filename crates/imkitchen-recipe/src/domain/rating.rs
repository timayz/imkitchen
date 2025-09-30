use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Star rating value object (1-5 stars)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Validate, PartialEq, Eq)]
pub struct StarRating {
    #[validate(range(min = 1, max = 5))]
    pub value: u8,
}

impl StarRating {
    pub fn new(rating: u8) -> Result<Self, validator::ValidationErrors> {
        let star_rating = Self { value: rating };
        star_rating.validate()?;
        Ok(star_rating)
    }

    pub fn one_star() -> Self {
        Self { value: 1 }
    }
    pub fn two_stars() -> Self {
        Self { value: 2 }
    }
    pub fn three_stars() -> Self {
        Self { value: 3 }
    }
    pub fn four_stars() -> Self {
        Self { value: 4 }
    }
    pub fn five_stars() -> Self {
        Self { value: 5 }
    }
}

/// Review moderation status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewModerationStatus {
    Pending,
    Approved,
    Flagged,
    Rejected,
}

impl std::fmt::Display for ReviewModerationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewModerationStatus::Pending => write!(f, "Pending"),
            ReviewModerationStatus::Approved => write!(f, "Approved"),
            ReviewModerationStatus::Flagged => write!(f, "Flagged"),
            ReviewModerationStatus::Rejected => write!(f, "Rejected"),
        }
    }
}

/// Helpfulness vote value object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HelpfulnessVote {
    pub user_id: Uuid,
    pub is_helpful: bool,
    pub voted_at: DateTime<Utc>,
}

impl HelpfulnessVote {
    pub fn new(user_id: Uuid, is_helpful: bool) -> Self {
        Self {
            user_id,
            is_helpful,
            voted_at: Utc::now(),
        }
    }
}

/// Recipe rating aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RecipeRating {
    pub rating_id: Uuid,
    pub user_id: Uuid,
    pub recipe_id: Uuid,
    pub star_rating: StarRating,
    pub created_at: DateTime<Utc>,
}

impl RecipeRating {
    pub fn new(
        user_id: Uuid,
        recipe_id: Uuid,
        star_rating: StarRating,
    ) -> Result<Self, validator::ValidationErrors> {
        let rating = Self {
            rating_id: Uuid::new_v4(),
            user_id,
            recipe_id,
            star_rating,
            created_at: Utc::now(),
        };
        rating.validate()?;
        Ok(rating)
    }
}

/// Recipe review aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RecipeReview {
    pub review_id: Uuid,
    pub rating_id: Uuid,
    pub user_id: Uuid,
    pub recipe_id: Uuid,
    #[validate(length(min = 10, max = 2000))]
    pub review_text: String,
    pub photos: Vec<String>,
    pub helpfulness_score: i32,
    pub helpfulness_votes: Vec<HelpfulnessVote>,
    pub moderation_status: ReviewModerationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RecipeReview {
    pub fn new(
        rating_id: Uuid,
        user_id: Uuid,
        recipe_id: Uuid,
        review_text: String,
        photos: Vec<String>,
    ) -> Result<Self, validator::ValidationErrors> {
        let review = Self {
            review_id: Uuid::new_v4(),
            rating_id,
            user_id,
            recipe_id,
            review_text,
            photos,
            helpfulness_score: 0,
            helpfulness_votes: Vec::new(),
            moderation_status: ReviewModerationStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        review.validate()?;
        Ok(review)
    }

    pub fn add_helpfulness_vote(&mut self, vote: HelpfulnessVote) -> Result<(), &'static str> {
        // Check if user has already voted
        if self
            .helpfulness_votes
            .iter()
            .any(|v| v.user_id == vote.user_id)
        {
            return Err("User has already voted on this review");
        }

        // Update helpfulness score
        if vote.is_helpful {
            self.helpfulness_score += 1;
        } else {
            self.helpfulness_score -= 1;
        }

        self.helpfulness_votes.push(vote);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_moderation_status(&mut self, status: ReviewModerationStatus) {
        self.moderation_status = status;
        self.updated_at = Utc::now();
    }

    pub fn edit_review_text(
        &mut self,
        new_text: String,
    ) -> Result<(), validator::ValidationErrors> {
        // Validate new text length
        if new_text.len() < 10 || new_text.len() > 2000 {
            let mut errors = validator::ValidationErrors::new();
            let error = validator::ValidationError::new("review_text_length");
            errors.add("review_text", error);
            return Err(errors);
        }

        self.review_text = new_text;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Rating statistics for recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingStatistics {
    pub recipe_id: Uuid,
    pub average_rating: f32,
    pub total_ratings: u32,
    pub rating_distribution: [u32; 5], // [1-star, 2-star, 3-star, 4-star, 5-star]
}

impl RatingStatistics {
    pub fn new(recipe_id: Uuid) -> Self {
        Self {
            recipe_id,
            average_rating: 0.0,
            total_ratings: 0,
            rating_distribution: [0; 5],
        }
    }

    pub fn add_rating(&mut self, rating: StarRating) {
        let rating_index = (rating.value - 1) as usize;
        self.rating_distribution[rating_index] += 1;
        self.total_ratings += 1;
        self.recalculate_average();
    }

    pub fn remove_rating(&mut self, rating: StarRating) {
        let rating_index = (rating.value - 1) as usize;
        if self.rating_distribution[rating_index] > 0 {
            self.rating_distribution[rating_index] -= 1;
            self.total_ratings -= 1;
            self.recalculate_average();
        }
    }

    fn recalculate_average(&mut self) {
        if self.total_ratings == 0 {
            self.average_rating = 0.0;
            return;
        }

        let weighted_sum: u32 = self
            .rating_distribution
            .iter()
            .enumerate()
            .map(|(index, &count)| (index as u32 + 1) * count)
            .sum();

        self.average_rating = weighted_sum as f32 / self.total_ratings as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_star_rating_validation() {
        assert!(StarRating::new(1).is_ok());
        assert!(StarRating::new(5).is_ok());
        assert!(StarRating::new(0).is_err());
        assert!(StarRating::new(6).is_err());
    }

    #[test]
    fn test_recipe_rating_creation() {
        let user_id = Uuid::new_v4();
        let recipe_id = Uuid::new_v4();
        let star_rating = StarRating::four_stars();

        let rating = RecipeRating::new(user_id, recipe_id, star_rating);
        assert!(rating.is_ok());

        let rating = rating.unwrap();
        assert_eq!(rating.user_id, user_id);
        assert_eq!(rating.recipe_id, recipe_id);
        assert_eq!(rating.star_rating, star_rating);
    }

    #[test]
    fn test_recipe_review_creation() {
        let rating_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let recipe_id = Uuid::new_v4();
        let review_text = "This recipe was absolutely delicious! Highly recommend.".to_string();
        let photos = vec!["photo1.jpg".to_string()];

        let review = RecipeReview::new(
            rating_id,
            user_id,
            recipe_id,
            review_text.clone(),
            photos.clone(),
        );
        assert!(review.is_ok());

        let review = review.unwrap();
        assert_eq!(review.rating_id, rating_id);
        assert_eq!(review.user_id, user_id);
        assert_eq!(review.recipe_id, recipe_id);
        assert_eq!(review.review_text, review_text);
        assert_eq!(review.photos, photos);
        assert_eq!(review.moderation_status, ReviewModerationStatus::Pending);
    }

    #[test]
    fn test_review_text_validation() {
        let rating_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let recipe_id = Uuid::new_v4();
        let photos = vec![];

        // Too short
        let short_text = "Good".to_string();
        let review = RecipeReview::new(rating_id, user_id, recipe_id, short_text, photos.clone());
        assert!(review.is_err());

        // Too long
        let long_text = "a".repeat(2001);
        let review = RecipeReview::new(rating_id, user_id, recipe_id, long_text, photos.clone());
        assert!(review.is_err());

        // Just right
        let good_text = "This recipe was really good and I enjoyed making it.".to_string();
        let review = RecipeReview::new(rating_id, user_id, recipe_id, good_text, photos);
        assert!(review.is_ok());
    }

    #[test]
    fn test_helpfulness_voting() {
        let mut review = RecipeReview::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Great recipe!".to_string(),
            vec![],
        )
        .unwrap();

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        // Add helpful vote
        let helpful_vote = HelpfulnessVote::new(user1, true);
        assert!(review.add_helpfulness_vote(helpful_vote).is_ok());
        assert_eq!(review.helpfulness_score, 1);

        // Add unhelpful vote
        let unhelpful_vote = HelpfulnessVote::new(user2, false);
        assert!(review.add_helpfulness_vote(unhelpful_vote).is_ok());
        assert_eq!(review.helpfulness_score, 0);

        // Try to vote again with same user - should fail
        let duplicate_vote = HelpfulnessVote::new(user1, false);
        assert!(review.add_helpfulness_vote(duplicate_vote).is_err());
    }

    #[test]
    fn test_rating_statistics() {
        let recipe_id = Uuid::new_v4();
        let mut stats = RatingStatistics::new(recipe_id);

        // Add some ratings
        stats.add_rating(StarRating::five_stars());
        stats.add_rating(StarRating::four_stars());
        stats.add_rating(StarRating::five_stars());

        assert_eq!(stats.total_ratings, 3);
        assert_eq!(stats.rating_distribution[4], 2); // 5-star ratings
        assert_eq!(stats.rating_distribution[3], 1); // 4-star ratings
        assert!((stats.average_rating - 4.67).abs() < 0.01); // (5+4+5)/3 = 4.67

        // Remove a rating
        stats.remove_rating(StarRating::four_stars());
        assert_eq!(stats.total_ratings, 2);
        assert_eq!(stats.rating_distribution[3], 0); // 4-star ratings
        assert_eq!(stats.average_rating, 5.0); // (5+5)/2 = 5.0
    }
}
