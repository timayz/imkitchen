// CQRS queries for recipe data

use crate::domain::{Difficulty, RecipeCategory};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeByIdQuery {
    pub recipe_id: Uuid,
}

impl RecipeByIdQuery {
    pub fn new(recipe_id: Uuid) -> Self {
        Self { recipe_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipesByUserQuery {
    pub user_id: Uuid,
    pub include_private: bool,
}

impl RecipesByUserQuery {
    pub fn new(user_id: Uuid, include_private: bool) -> Self {
        Self {
            user_id,
            include_private,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSearchQuery {
    pub search_text: Option<String>,
    pub category: Option<RecipeCategory>,
    pub difficulty: Option<Difficulty>,
    pub max_prep_time: Option<u32>,
    pub max_cook_time: Option<u32>,
    pub tags: Vec<String>,
    pub user_id: Option<Uuid>, // For user-specific searches
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl RecipeSearchQuery {
    pub fn new() -> Self {
        Self {
            search_text: None,
            category: None,
            difficulty: None,
            max_prep_time: None,
            max_cook_time: None,
            tags: Vec::new(),
            user_id: None,
            limit: Some(20), // Default limit
            offset: Some(0),
        }
    }

    pub fn with_search_text(mut self, text: String) -> Self {
        self.search_text = Some(text);
        self
    }

    pub fn with_category(mut self, category: RecipeCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = Some(difficulty);
        self
    }

    pub fn with_max_prep_time(mut self, max_prep_time: u32) -> Self {
        self.max_prep_time = Some(max_prep_time);
        self
    }

    pub fn with_max_cook_time(mut self, max_cook_time: u32) -> Self {
        self.max_cook_time = Some(max_cook_time);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_user_context(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

impl Default for RecipeSearchQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipesByTagQuery {
    pub tag: String,
    pub user_id: Option<Uuid>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl RecipesByTagQuery {
    pub fn new(tag: String) -> Self {
        Self {
            tag,
            user_id: None,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn with_user_context(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularRecipesQuery {
    pub category: Option<RecipeCategory>,
    pub time_range_days: Option<u32>, // For trending recipes
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl PopularRecipesQuery {
    pub fn new() -> Self {
        Self {
            category: None,
            time_range_days: None,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn with_category(mut self, category: RecipeCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_time_range(mut self, days: u32) -> Self {
        self.time_range_days = Some(days);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

impl Default for PopularRecipesQuery {
    fn default() -> Self {
        Self::new()
    }
}

// Collection queries

use crate::domain::collection::CollectionPrivacy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionByIdQuery {
    pub collection_id: Uuid,
    pub user_id: Option<Uuid>, // For access control
}

impl CollectionByIdQuery {
    pub fn new(collection_id: Uuid) -> Self {
        Self {
            collection_id,
            user_id: None,
        }
    }

    pub fn with_user_context(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionsByUserQuery {
    pub user_id: Uuid,
    pub include_archived: bool,
    pub privacy_filter: Option<CollectionPrivacy>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl CollectionsByUserQuery {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            include_archived: false,
            privacy_filter: None,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn include_archived(mut self) -> Self {
        self.include_archived = true;
        self
    }

    pub fn filter_by_privacy(mut self, privacy: CollectionPrivacy) -> Self {
        self.privacy_filter = Some(privacy);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipesByCollectionQuery {
    pub collection_id: Uuid,
    pub user_id: Option<Uuid>, // For access control
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl RecipesByCollectionQuery {
    pub fn new(collection_id: Uuid) -> Self {
        Self {
            collection_id,
            user_id: None,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn with_user_context(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSearchQuery {
    pub search_text: Option<String>,
    pub privacy: Option<CollectionPrivacy>,
    pub user_id: Option<Uuid>, // For user-specific searches or access control
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl CollectionSearchQuery {
    pub fn new() -> Self {
        Self {
            search_text: None,
            privacy: None,
            user_id: None,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn with_search_text(mut self, text: String) -> Self {
        self.search_text = Some(text);
        self
    }

    pub fn filter_by_privacy(mut self, privacy: CollectionPrivacy) -> Self {
        self.privacy = Some(privacy);
        self
    }

    pub fn with_user_context(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

impl Default for CollectionSearchQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInCollectionsQuery {
    pub recipe_id: Uuid,
    pub user_id: Option<Uuid>, // For access control
}

impl RecipeInCollectionsQuery {
    pub fn new(recipe_id: Uuid) -> Self {
        Self {
            recipe_id,
            user_id: None,
        }
    }

    pub fn with_user_context(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFavoritesQuery {
    pub user_id: Uuid,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl UserFavoritesQuery {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

// Rating and Review queries

use crate::domain::rating::ReviewModerationStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingsByRecipeQuery {
    pub recipe_id: Uuid,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl RatingsByRecipeQuery {
    pub fn new(recipe_id: Uuid) -> Self {
        Self {
            recipe_id,
            limit: Some(50),
            offset: Some(0),
        }
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewsByRecipeQuery {
    pub recipe_id: Uuid,
    pub moderation_status_filter: Option<ReviewModerationStatus>,
    pub sort_by_helpfulness: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl ReviewsByRecipeQuery {
    pub fn new(recipe_id: Uuid) -> Self {
        Self {
            recipe_id,
            moderation_status_filter: Some(ReviewModerationStatus::Approved),
            sort_by_helpfulness: true,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn include_all_moderation_statuses(mut self) -> Self {
        self.moderation_status_filter = None;
        self
    }

    pub fn filter_by_moderation_status(mut self, status: ReviewModerationStatus) -> Self {
        self.moderation_status_filter = Some(status);
        self
    }

    pub fn sort_by_date_instead(mut self) -> Self {
        self.sort_by_helpfulness = false;
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRatingHistoryQuery {
    pub user_id: Uuid,
    pub recipe_id: Option<Uuid>, // Optional filter for specific recipe
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl UserRatingHistoryQuery {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            recipe_id: None,
            limit: Some(50),
            offset: Some(0),
        }
    }

    pub fn for_recipe(mut self, recipe_id: Uuid) -> Self {
        self.recipe_id = Some(recipe_id);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReviewHistoryQuery {
    pub user_id: Uuid,
    pub moderation_status_filter: Option<ReviewModerationStatus>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl UserReviewHistoryQuery {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            moderation_status_filter: None,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn filter_by_moderation_status(mut self, status: ReviewModerationStatus) -> Self {
        self.moderation_status_filter = Some(status);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeRatingStatsQuery {
    pub recipe_id: Uuid,
}

impl RecipeRatingStatsQuery {
    pub fn new(recipe_id: Uuid) -> Self {
        Self { recipe_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewsByModerationStatusQuery {
    pub moderation_status: ReviewModerationStatus,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl ReviewsByModerationStatusQuery {
    pub fn new(moderation_status: ReviewModerationStatus) -> Self {
        Self {
            moderation_status,
            limit: Some(50),
            offset: Some(0),
        }
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopRatedRecipesQuery {
    pub category: Option<RecipeCategory>,
    pub min_rating: f32,
    pub min_review_count: u32,
    pub time_range_days: Option<u32>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl TopRatedRecipesQuery {
    pub fn new() -> Self {
        Self {
            category: None,
            min_rating: 4.0,
            min_review_count: 5,
            time_range_days: None,
            limit: Some(20),
            offset: Some(0),
        }
    }

    pub fn with_category(mut self, category: RecipeCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_minimum_rating(mut self, min_rating: f32) -> Self {
        self.min_rating = min_rating;
        self
    }

    pub fn with_minimum_reviews(mut self, min_review_count: u32) -> Self {
        self.min_review_count = min_review_count;
        self
    }

    pub fn with_time_range(mut self, days: u32) -> Self {
        self.time_range_days = Some(days);
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

impl Default for TopRatedRecipesQuery {
    fn default() -> Self {
        Self::new()
    }
}
