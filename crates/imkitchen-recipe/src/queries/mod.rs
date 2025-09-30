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
