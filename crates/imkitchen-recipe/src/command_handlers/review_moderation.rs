use crate::commands::{FlagReviewCommand, ModerateReviewCommand};
use crate::domain::rating::{RecipeReview, ReviewModerationStatus};
use crate::domain::services::ReviewModerationService;
use crate::events::{ReviewFlagged, ReviewModerated};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Result type for moderation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationResult {
    pub review_id: Uuid,
    pub previous_status: ReviewModerationStatus,
    pub new_status: ReviewModerationStatus,
    pub moderation_reason: Option<String>,
    pub auto_approved: bool,
}

/// Command handler for review moderation operations
pub struct ReviewModerationCommandHandler {
    moderation_service: Arc<ReviewModerationService>,
}

impl ReviewModerationCommandHandler {
    pub fn new(moderation_service: Arc<ReviewModerationService>) -> Self {
        Self { moderation_service }
    }

    /// Handle manual review moderation by admin/moderator
    pub async fn handle_moderate_review(
        &self,
        command: ModerateReviewCommand,
        current_review: RecipeReview,
    ) -> Result<(RecipeReview, Vec<ReviewModerated>), String> {
        // Validate that the user has moderation permissions (would be checked by auth middleware)

        let mut updated_review = current_review.clone();
        let _previous_status = updated_review.moderation_status;

        // Update moderation status
        updated_review.moderation_status = command.moderation_status;
        updated_review.updated_at = Utc::now();

        // Generate moderation event
        let moderation_event = ReviewModerated {
            event_id: Uuid::new_v4(),
            review_id: command.review_id,
            moderation_status: command.moderation_status,
            moderation_reason: command.moderation_reason,
            occurred_at: Utc::now(),
        };

        Ok((updated_review, vec![moderation_event]))
    }

    /// Handle automatic review moderation
    pub async fn handle_auto_moderate_review(
        &self,
        review: &RecipeReview,
    ) -> Result<(ReviewModerationStatus, Option<String>), String> {
        let moderation_status = self.moderation_service.moderate_review(review);

        let reason = match moderation_status {
            ReviewModerationStatus::Approved => {
                Some("Auto-approved: passed content filters".to_string())
            }
            ReviewModerationStatus::Pending => Some("Pending: requires manual review".to_string()),
            ReviewModerationStatus::Flagged => {
                Some("Flagged: potential spam content detected".to_string())
            }
            ReviewModerationStatus::Rejected => {
                Some("Rejected: inappropriate content detected".to_string())
            }
        };

        Ok((moderation_status, reason))
    }

    /// Handle review flagging by users
    pub async fn handle_flag_review(
        &self,
        command: FlagReviewCommand,
        current_review: RecipeReview,
    ) -> Result<(RecipeReview, Vec<ReviewFlagged>), String> {
        // Validate that the user isn't flagging their own review
        if current_review.user_id == command.flagged_by {
            return Err("Users cannot flag their own reviews".to_string());
        }

        let mut updated_review = current_review.clone();

        // If review was previously approved, flag it for manual review
        if updated_review.moderation_status == ReviewModerationStatus::Approved {
            updated_review.moderation_status = ReviewModerationStatus::Flagged;
            updated_review.updated_at = Utc::now();
        }

        // Generate flag event
        let flag_event = ReviewFlagged {
            event_id: Uuid::new_v4(),
            review_id: command.review_id,
            flagged_by: command.flagged_by,
            flag_reason: command.flag_reason,
            occurred_at: Utc::now(),
        };

        Ok((updated_review, vec![flag_event]))
    }

    /// Bulk moderation operation for admin efficiency
    pub async fn handle_bulk_moderate_reviews(
        &self,
        review_ids: Vec<Uuid>,
        moderation_status: ReviewModerationStatus,
        moderation_reason: Option<String>,
        _moderated_by: Uuid,
        reviews: Vec<RecipeReview>,
    ) -> Result<(Vec<RecipeReview>, Vec<ReviewModerated>), String> {
        let mut updated_reviews = Vec::new();
        let mut events = Vec::new();

        for review in reviews {
            if !review_ids.contains(&review.review_id) {
                continue;
            }

            let mut updated_review = review.clone();
            let _previous_status = updated_review.moderation_status;

            updated_review.moderation_status = moderation_status;
            updated_review.updated_at = Utc::now();

            let moderation_event = ReviewModerated {
                event_id: Uuid::new_v4(),
                review_id: review.review_id,
                moderation_status,
                moderation_reason: moderation_reason.clone(),
                occurred_at: Utc::now(),
            };

            updated_reviews.push(updated_review);
            events.push(moderation_event);
        }

        Ok((updated_reviews, events))
    }

    /// Check if review should be auto-approved based on user reputation and content
    pub async fn should_auto_approve(
        &self,
        review: &RecipeReview,
        user_reputation_score: Option<f32>,
    ) -> bool {
        // Check content first
        let moderation_status = self.moderation_service.moderate_review(review);
        if moderation_status != ReviewModerationStatus::Approved {
            return false;
        }

        // Check if user has good reputation (in real implementation, this would come from user service)
        let user_score = user_reputation_score.unwrap_or(0.5);
        if user_score < 0.7 {
            return false;
        }

        // Additional quality checks
        !self.moderation_service.requires_manual_review(review)
    }
}

impl Default for ReviewModerationCommandHandler {
    fn default() -> Self {
        Self::new(Arc::new(ReviewModerationService::new()))
    }
}

/// Service for managing moderation queue and statistics
pub struct ModerationQueueService {
    moderation_service: Arc<ReviewModerationService>,
}

impl ModerationQueueService {
    pub fn new(moderation_service: Arc<ReviewModerationService>) -> Self {
        Self { moderation_service }
    }

    /// Get moderation queue statistics
    pub async fn get_moderation_stats(&self, reviews: &[RecipeReview]) -> ModerationQueueStats {
        let pending_count = reviews
            .iter()
            .filter(|r| r.moderation_status == ReviewModerationStatus::Pending)
            .count() as u32;

        let flagged_count = reviews
            .iter()
            .filter(|r| r.moderation_status == ReviewModerationStatus::Flagged)
            .count() as u32;

        let approved_count = reviews
            .iter()
            .filter(|r| r.moderation_status == ReviewModerationStatus::Approved)
            .count() as u32;

        let rejected_count = reviews
            .iter()
            .filter(|r| r.moderation_status == ReviewModerationStatus::Rejected)
            .count() as u32;

        let auto_approved_count = reviews
            .iter()
            .filter(|r| r.moderation_status == ReviewModerationStatus::Approved)
            .count() as u32;

        ModerationQueueStats {
            total_reviews: reviews.len() as u32,
            pending_count,
            flagged_count,
            approved_count,
            rejected_count,
            auto_approved_count,
            manual_review_required: pending_count + flagged_count,
        }
    }

    /// Get priority reviews that need immediate attention
    pub async fn get_priority_reviews<'a>(
        &self,
        reviews: &'a [RecipeReview],
        limit: usize,
    ) -> Vec<&'a RecipeReview> {
        let mut priority_reviews: Vec<&RecipeReview> = reviews
            .iter()
            .filter(|r| {
                r.moderation_status == ReviewModerationStatus::Flagged
                    || (r.moderation_status == ReviewModerationStatus::Pending
                        && self.moderation_service.requires_manual_review(r))
            })
            .collect();

        // Sort by creation time (oldest first for queue fairness)
        priority_reviews.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        priority_reviews.into_iter().take(limit).collect()
    }

    /// Get reviews by moderation status with pagination
    pub async fn get_reviews_by_status<'a>(
        &self,
        reviews: &'a [RecipeReview],
        status: ReviewModerationStatus,
        page: u32,
        limit: u32,
    ) -> Vec<&'a RecipeReview> {
        let offset = (page - 1) * limit;

        let mut filtered_reviews: Vec<&RecipeReview> = reviews
            .iter()
            .filter(|r| r.moderation_status == status)
            .collect();

        // Sort by creation time (newest first for most statuses, oldest first for pending)
        match status {
            ReviewModerationStatus::Pending | ReviewModerationStatus::Flagged => {
                filtered_reviews.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            _ => {
                filtered_reviews.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
        }

        filtered_reviews
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect()
    }
}

impl Default for ModerationQueueService {
    fn default() -> Self {
        Self::new(Arc::new(ReviewModerationService::new()))
    }
}

/// Statistics for the moderation queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationQueueStats {
    pub total_reviews: u32,
    pub pending_count: u32,
    pub flagged_count: u32,
    pub approved_count: u32,
    pub rejected_count: u32,
    pub auto_approved_count: u32,
    pub manual_review_required: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rating::{HelpfulnessVote, StarRating};
    use std::sync::Arc;

    fn create_test_review() -> RecipeReview {
        RecipeReview {
            review_id: Uuid::new_v4(),
            recipe_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating_id: Uuid::new_v4(),
            review_text: "This is a great recipe with detailed instructions.".to_string(),
            photos: vec![],
            moderation_status: ReviewModerationStatus::Pending,
            helpfulness_score: 0,
            helpfulness_votes: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_auto_moderation() {
        let handler = ReviewModerationCommandHandler::default();
        let review = create_test_review();

        let result = handler.handle_auto_moderate_review(&review).await;
        assert!(result.is_ok());

        let (status, reason) = result.unwrap();
        assert_eq!(status, ReviewModerationStatus::Approved);
        assert!(reason.is_some());
    }

    #[tokio::test]
    async fn test_manual_moderation() {
        let handler = ReviewModerationCommandHandler::default();
        let review = create_test_review();
        let moderator_id = Uuid::new_v4();

        let command = ModerateReviewCommand {
            review_id: review.review_id,
            moderation_status: ReviewModerationStatus::Approved,
            moderation_reason: Some("Manually approved".to_string()),
            moderated_by: moderator_id,
        };

        let result = handler.handle_moderate_review(command, review).await;
        assert!(result.is_ok());

        let (updated_review, events) = result.unwrap();
        assert_eq!(
            updated_review.moderation_status,
            ReviewModerationStatus::Approved
        );
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_flag_review() {
        let handler = ReviewModerationCommandHandler::default();
        let mut review = create_test_review();
        review.moderation_status = ReviewModerationStatus::Approved;

        let command = FlagReviewCommand::new(
            review.review_id,
            Uuid::new_v4(), // Different user
            "Inappropriate content".to_string(),
        )
        .unwrap();

        let result = handler.handle_flag_review(command, review).await;
        assert!(result.is_ok());

        let (updated_review, events) = result.unwrap();
        assert_eq!(
            updated_review.moderation_status,
            ReviewModerationStatus::Flagged
        );
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_cannot_flag_own_review() {
        let handler = ReviewModerationCommandHandler::default();
        let review = create_test_review();

        let command = FlagReviewCommand::new(
            review.review_id,
            review.user_id, // Same user
            "Test reason".to_string(),
        )
        .unwrap();

        let result = handler.handle_flag_review(command, review).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot flag their own"));
    }

    #[tokio::test]
    async fn test_moderation_queue_stats() {
        let service = ModerationQueueService::default();
        let reviews = vec![
            {
                let mut r = create_test_review();
                r.moderation_status = ReviewModerationStatus::Pending;
                r
            },
            {
                let mut r = create_test_review();
                r.moderation_status = ReviewModerationStatus::Approved;
                r
            },
            {
                let mut r = create_test_review();
                r.moderation_status = ReviewModerationStatus::Flagged;
                r
            },
        ];

        let stats = service.get_moderation_stats(&reviews).await;
        assert_eq!(stats.total_reviews, 3);
        assert_eq!(stats.pending_count, 1);
        assert_eq!(stats.approved_count, 1);
        assert_eq!(stats.flagged_count, 1);
        assert_eq!(stats.manual_review_required, 2);
    }
}
