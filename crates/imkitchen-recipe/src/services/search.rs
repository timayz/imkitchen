use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::domain::{Recipe, RecipeCategory, Difficulty};
use imkitchen_shared::types::{DietaryRestriction, MealType};

/// Advanced search service with suggestion algorithms and similarity matching
#[derive(Debug, Clone)]
pub struct RecipeSearchService {
    // In a real implementation, this would have database connections
    // For now, we'll use mock data and focus on algorithm implementation
}

/// Search suggestion with scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    pub text: String,
    pub suggestion_type: SuggestionType,
    pub score: f64,
    pub search_count: u32,
    pub success_rate: f64,
}

/// Types of search suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Ingredient,
    Recipe,
    Tag,
    Category,
    Autocomplete,
    TypoCorrection,
}

/// Recipe similarity result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSimilarity {
    pub recipe_id: Uuid,
    pub similarity_score: f64,
    pub similarity_reasons: Vec<SimilarityReason>,
}

/// Reasons why recipes are similar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimilarityReason {
    IngredientOverlap { common_ingredients: Vec<String>, overlap_score: f64 },
    CookingTechnique { common_techniques: Vec<String>, technique_score: f64 },
    Category { category: RecipeCategory },
    Difficulty { difficulty: Difficulty },
    PrepTime { similar_time_range: String },
}

/// Search analytics for query optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnalytics {
    pub query: String,
    pub results_count: u32,
    pub click_through_rate: f64,
    pub search_duration_ms: u64,
    pub popular_filters: HashMap<String, u32>,
    pub successful_suggestions: Vec<String>,
}

/// User search preferences for personalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchPreferences {
    pub user_id: Uuid,
    pub preferred_categories: Vec<RecipeCategory>,
    pub preferred_difficulty: Vec<Difficulty>,
    pub max_prep_time: Option<u32>,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub meal_types: Vec<MealType>,
    pub search_history_weight: f64,
    pub recent_searches: Vec<String>,
}

impl RecipeSearchService {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate search suggestions based on query and history
    pub async fn generate_suggestions(
        &self,
        partial_query: &str,
        user_preferences: Option<&UserSearchPreferences>,
        limit: usize,
    ) -> Result<Vec<SearchSuggestion>, SearchError> {
        let mut suggestions = Vec::new();
        
        // Ingredient-based suggestions
        suggestions.extend(self.suggest_ingredients(partial_query, limit / 4).await?);
        
        // Recipe name suggestions
        suggestions.extend(self.suggest_recipes(partial_query, limit / 4).await?);
        
        // Tag and category suggestions
        suggestions.extend(self.suggest_tags_and_categories(partial_query, limit / 4).await?);
        
        // Typo correction suggestions
        suggestions.extend(self.suggest_typo_corrections(partial_query, limit / 4).await?);
        
        // Apply user preference weighting
        if let Some(prefs) = user_preferences {
            self.apply_preference_weighting(&mut suggestions, prefs);
        }
        
        // Sort by score and limit results
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(limit);
        
        Ok(suggestions)
    }

    /// Find similar recipes using multiple algorithms
    pub async fn find_similar_recipes(
        &self,
        recipe_id: Uuid,
        limit: usize,
    ) -> Result<Vec<RecipeSimilarity>, SearchError> {
        // Get the base recipe (in real implementation, from database)
        let base_recipe = self.get_recipe_by_id(recipe_id).await?;
        
        let mut similarities = Vec::new();
        
        // Find recipes with ingredient overlap
        let ingredient_similar = self.find_ingredient_similar(&base_recipe, limit * 2).await?;
        similarities.extend(ingredient_similar);
        
        // Find recipes with similar cooking techniques
        let technique_similar = self.find_technique_similar(&base_recipe, limit * 2).await?;
        similarities.extend(technique_similar);
        
        // Find recipes in same category with similar attributes
        let category_similar = self.find_category_similar(&base_recipe, limit).await?;
        similarities.extend(category_similar);
        
        // Deduplicate and merge similarity scores
        let mut merged = self.merge_similarity_scores(similarities);
        
        // Sort by combined score and limit
        merged.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));
        merged.truncate(limit);
        
        Ok(merged)
    }

    /// Calculate ingredient overlap similarity
    async fn find_ingredient_similar(
        &self,
        base_recipe: &Recipe,
        limit: usize,
    ) -> Result<Vec<RecipeSimilarity>, SearchError> {
        // Mock implementation - in real version, this would query database
        // and calculate actual ingredient overlap scores
        
        let base_ingredients: HashSet<String> = base_recipe.ingredients.iter()
            .map(|ing| ing.name.to_lowercase())
            .collect();
        
        // Get candidate recipes (mock data for now)
        let candidates = self.get_candidate_recipes(&base_recipe.category, limit * 3).await?;
        
        let mut similarities = Vec::new();
        
        for candidate in candidates {
            if candidate.recipe_id == base_recipe.recipe_id {
                continue; // Skip self
            }
            
            let candidate_ingredients: HashSet<String> = candidate.ingredients.iter()
                .map(|ing| ing.name.to_lowercase())
                .collect();
            
            let intersection: HashSet<_> = base_ingredients.intersection(&candidate_ingredients).collect();
            let union: HashSet<_> = base_ingredients.union(&candidate_ingredients).collect();
            
            if intersection.len() > 0 {
                let jaccard_score = intersection.len() as f64 / union.len() as f64;
                
                // Boost score if many core ingredients match
                let core_ingredient_bonus = if intersection.len() >= 3 { 0.2 } else { 0.0 };
                let final_score = (jaccard_score + core_ingredient_bonus).min(1.0);
                
                if final_score > 0.1 { // Minimum threshold
                    similarities.push(RecipeSimilarity {
                        recipe_id: candidate.recipe_id,
                        similarity_score: final_score,
                        similarity_reasons: vec![
                            SimilarityReason::IngredientOverlap {
                                common_ingredients: intersection.into_iter().cloned().collect(),
                                overlap_score: jaccard_score,
                            }
                        ],
                    });
                }
            }
        }
        
        Ok(similarities)
    }

    /// Find recipes with similar cooking techniques
    async fn find_technique_similar(
        &self,
        base_recipe: &Recipe,
        limit: usize,
    ) -> Result<Vec<RecipeSimilarity>, SearchError> {
        // Extract cooking techniques from instructions using simple keyword matching
        let techniques = self.extract_cooking_techniques(&base_recipe.instructions);
        
        if techniques.is_empty() {
            return Ok(Vec::new());
        }
        
        let candidates = self.get_candidate_recipes(&base_recipe.category, limit * 2).await?;
        let mut similarities = Vec::new();
        
        for candidate in candidates {
            if candidate.recipe_id == base_recipe.recipe_id {
                continue;
            }
            
            let candidate_techniques = self.extract_cooking_techniques(&candidate.instructions);
            let common_techniques: Vec<String> = techniques.intersection(&candidate_techniques)
                .cloned()
                .collect();
            
            if !common_techniques.is_empty() {
                let technique_score = common_techniques.len() as f64 / 
                    techniques.union(&candidate_techniques).count() as f64;
                
                if technique_score > 0.2 {
                    similarities.push(RecipeSimilarity {
                        recipe_id: candidate.recipe_id,
                        similarity_score: technique_score * 0.7, // Weight technique similarity lower than ingredient
                        similarity_reasons: vec![
                            SimilarityReason::CookingTechnique {
                                common_techniques,
                                technique_score,
                            }
                        ],
                    });
                }
            }
        }
        
        Ok(similarities)
    }

    /// Find recipes in same category with similar attributes
    async fn find_category_similar(
        &self,
        base_recipe: &Recipe,
        limit: usize,
    ) -> Result<Vec<RecipeSimilarity>, SearchError> {
        let candidates = self.get_candidate_recipes(&base_recipe.category, limit * 2).await?;
        let mut similarities = Vec::new();
        
        for candidate in candidates {
            if candidate.recipe_id == base_recipe.recipe_id {
                continue;
            }
            
            let mut score = 0.3; // Base category match score
            let mut reasons = vec![SimilarityReason::Category { 
                category: base_recipe.category.clone() 
            }];
            
            // Same difficulty bonus
            if candidate.difficulty == base_recipe.difficulty {
                score += 0.1;
                reasons.push(SimilarityReason::Difficulty { 
                    difficulty: base_recipe.difficulty.clone() 
                });
            }
            
            // Similar prep time bonus
            let time_diff = (candidate.prep_time_minutes as i32 - base_recipe.prep_time_minutes as i32).abs();
            if time_diff <= 15 {
                score += 0.1;
                reasons.push(SimilarityReason::PrepTime { 
                    similar_time_range: format!("±15 minutes") 
                });
            }
            
            if score > 0.3 {
                similarities.push(RecipeSimilarity {
                    recipe_id: candidate.recipe_id,
                    similarity_score: score,
                    similarity_reasons: reasons,
                });
            }
        }
        
        Ok(similarities)
    }

    /// Extract cooking techniques from instruction text
    fn extract_cooking_techniques(&self, instructions: &[crate::domain::Instruction]) -> HashSet<String> {
        let techniques = [
            "sauté", "sautee", "fry", "deep fry", "pan fry",
            "bake", "roast", "grill", "broil",
            "boil", "simmer", "poach", "steam",
            "braise", "stew", "slow cook",
            "whisk", "whip", "beat", "fold", "mix",
            "chop", "dice", "mince", "slice", "julienne",
            "marinate", "season", "salt", "pepper",
            "reduce", "deglaze", "caramelize",
            "rest", "chill", "freeze", "thaw",
        ];
        
        let instruction_text = instructions.iter()
            .map(|inst| inst.text.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
        
        techniques.iter()
            .filter(|&technique| instruction_text.contains(technique))
            .map(|&technique| technique.to_string())
            .collect()
    }

    /// Merge overlapping similarity scores for the same recipe
    fn merge_similarity_scores(&self, similarities: Vec<RecipeSimilarity>) -> Vec<RecipeSimilarity> {
        let mut merged: HashMap<Uuid, RecipeSimilarity> = HashMap::new();
        
        for similarity in similarities {
            match merged.get_mut(&similarity.recipe_id) {
                Some(existing) => {
                    // Combine scores using weighted average
                    existing.similarity_score = (existing.similarity_score + similarity.similarity_score) / 2.0;
                    existing.similarity_reasons.extend(similarity.similarity_reasons);
                }
                None => {
                    merged.insert(similarity.recipe_id, similarity);
                }
            }
        }
        
        merged.into_values().collect()
    }

    /// Generate ingredient-based suggestions
    async fn suggest_ingredients(&self, query: &str, limit: usize) -> Result<Vec<SearchSuggestion>, SearchError> {
        // Common ingredients database (in real implementation, this would be from DB)
        let ingredients = [
            "chicken", "beef", "pork", "fish", "salmon", "tuna",
            "rice", "pasta", "noodles", "bread", "flour",
            "tomato", "onion", "garlic", "potato", "carrot",
            "cheese", "milk", "eggs", "butter", "oil",
            "salt", "pepper", "herbs", "spices",
        ];
        
        let query_lower = query.to_lowercase();
        let mut suggestions = Vec::new();
        
        for &ingredient in &ingredients {
            if ingredient.contains(&query_lower) && suggestions.len() < limit {
                let score = if ingredient.starts_with(&query_lower) { 0.9 } else { 0.6 };
                suggestions.push(SearchSuggestion {
                    text: ingredient.to_string(),
                    suggestion_type: SuggestionType::Ingredient,
                    score,
                    search_count: 100, // Mock data
                    success_rate: 0.75,
                });
            }
        }
        
        Ok(suggestions)
    }

    /// Generate recipe name suggestions  
    async fn suggest_recipes(&self, query: &str, limit: usize) -> Result<Vec<SearchSuggestion>, SearchError> {
        // Mock recipe suggestions
        let recipes = [
            "Chicken Parmesan", "Beef Stir Fry", "Salmon Teriyaki",
            "Pasta Carbonara", "Vegetable Curry", "Chocolate Cake",
            "Caesar Salad", "Mushroom Risotto", "Fish Tacos",
        ];
        
        let query_lower = query.to_lowercase();
        let mut suggestions = Vec::new();
        
        for &recipe in &recipes {
            if recipe.to_lowercase().contains(&query_lower) && suggestions.len() < limit {
                let score = if recipe.to_lowercase().starts_with(&query_lower) { 0.95 } else { 0.7 };
                suggestions.push(SearchSuggestion {
                    text: recipe.to_string(),
                    suggestion_type: SuggestionType::Recipe,
                    score,
                    search_count: 50,
                    success_rate: 0.85,
                });
            }
        }
        
        Ok(suggestions)
    }

    /// Generate tag and category suggestions
    async fn suggest_tags_and_categories(&self, query: &str, limit: usize) -> Result<Vec<SearchSuggestion>, SearchError> {
        let tags = [
            "quick", "easy", "healthy", "vegetarian", "vegan", "gluten-free",
            "low-carb", "high-protein", "comfort-food", "spicy", "mild",
            "breakfast", "lunch", "dinner", "dessert", "snack",
        ];
        
        let query_lower = query.to_lowercase();
        let mut suggestions = Vec::new();
        
        for &tag in &tags {
            if tag.contains(&query_lower) && suggestions.len() < limit {
                let score = if tag.starts_with(&query_lower) { 0.8 } else { 0.5 };
                suggestions.push(SearchSuggestion {
                    text: tag.to_string(),
                    suggestion_type: SuggestionType::Tag,
                    score,
                    search_count: 25,
                    success_rate: 0.65,
                });
            }
        }
        
        Ok(suggestions)
    }

    /// Generate typo correction suggestions
    async fn suggest_typo_corrections(&self, query: &str, limit: usize) -> Result<Vec<SearchSuggestion>, SearchError> {
        // Simple typo correction using edit distance
        let common_terms = [
            "chicken", "recipe", "quick", "easy", "healthy", "delicious",
            "pasta", "sauce", "cheese", "vegetables", "cooking",
        ];
        
        let mut suggestions = Vec::new();
        
        for &term in &common_terms {
            if suggestions.len() >= limit {
                break;
            }
            
            let distance = self.levenshtein_distance(query, term);
            if distance <= 2 && distance > 0 && query.len() > 2 {
                let score = 1.0 - (distance as f64 / query.len().max(term.len()) as f64);
                if score > 0.6 {
                    suggestions.push(SearchSuggestion {
                        text: term.to_string(),
                        suggestion_type: SuggestionType::TypoCorrection,
                        score: score * 0.8, // Weight lower than exact matches
                        search_count: 10,
                        success_rate: 0.4,
                    });
                }
            }
        }
        
        Ok(suggestions)
    }

    /// Apply user preference weighting to suggestions
    fn apply_preference_weighting(&self, suggestions: &mut [SearchSuggestion], prefs: &UserSearchPreferences) {
        for suggestion in suggestions {
            // Boost suggestions that match user's recent searches
            if prefs.recent_searches.iter().any(|search| {
                search.to_lowercase().contains(&suggestion.text.to_lowercase())
            }) {
                suggestion.score *= 1.2;
            }
            
            // Apply search history weighting
            suggestion.score = suggestion.score * (1.0 - prefs.search_history_weight) + 
                              suggestion.score * prefs.search_history_weight * suggestion.success_rate;
        }
    }

    /// Calculate Levenshtein distance for typo correction
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        matrix[len1][len2]
    }

    /// Record search analytics for optimization
    pub async fn record_search_analytics(&self, analytics: SearchAnalytics) -> Result<(), SearchError> {
        // In real implementation, this would store analytics in the database
        // for query optimization and suggestion improvement
        println!("Recording search analytics: {:?}", analytics.query);
        Ok(())
    }

    /// Update user search preferences based on behavior
    pub async fn update_user_preferences(
        &self, 
        user_id: Uuid, 
        search_query: &str,
        _clicked_recipe_id: Option<Uuid>,
    ) -> Result<(), SearchError> {
        // In real implementation, this would update user preferences
        // based on search behavior and successful clicks
        println!("Updating preferences for user {} based on search: {}", user_id, search_query);
        Ok(())
    }

    // Mock helper methods (in real implementation, these would query the database)
    
    async fn get_recipe_by_id(&self, recipe_id: Uuid) -> Result<Recipe, SearchError> {
        // Mock recipe for demonstration
        use crate::domain::{RecipeParams, Ingredient, Instruction};
        
        let ingredients = vec![
            Ingredient::new("Chicken".to_string(), 1.0, "lb".to_string(), None)?,
            Ingredient::new("Rice".to_string(), 2.0, "cups".to_string(), None)?,
        ];
        
        let instructions = vec![
            Instruction::new(1, "Cook the chicken".to_string(), Some(20))?,
            Instruction::new(2, "Add rice and simmer".to_string(), Some(15))?,
        ];
        
        Recipe::new(RecipeParams {
            title: "Sample Recipe".to_string(),
            ingredients,
            instructions,
            prep_time_minutes: 15,
            cook_time_minutes: 30,
            difficulty: Difficulty::Medium,
            category: RecipeCategory::Main,
            created_by: recipe_id, // Mock user ID
            is_public: true,
            tags: vec!["quick".to_string(), "easy".to_string()],
        }).map_err(|e| SearchError::DomainError(e.to_string()))
    }
    
    async fn get_candidate_recipes(&self, _category: &RecipeCategory, limit: usize) -> Result<Vec<Recipe>, SearchError> {
        // Return mock recipes for demonstration
        let mut recipes = Vec::new();
        for _i in 0..limit.min(5) {
            let recipe = self.get_recipe_by_id(Uuid::new_v4()).await?;
            recipes.push(recipe);
        }
        Ok(recipes)
    }
}

impl Default for RecipeSearchService {
    fn default() -> Self {
        Self::new()
    }
}

/// Search service errors
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Invalid search query: {0}")]
    InvalidQuery(String),
    
    #[error("Search timeout")]
    Timeout,
    
    #[error("Recipe not found: {0}")]
    RecipeNotFound(Uuid),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    
    #[error("Domain error: {0}")]
    DomainError(String),
}

// Note: Domain errors will be handled through string conversion for now

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_suggestions() {
        let service = RecipeSearchService::new();
        let suggestions = service.generate_suggestions("chick", None, 10).await.unwrap();
        
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.text.contains("chicken")));
    }

    #[tokio::test]
    async fn test_find_similar_recipes() {
        let service = RecipeSearchService::new();
        let recipe_id = Uuid::new_v4();
        let similarities = service.find_similar_recipes(recipe_id, 5).await.unwrap();
        
        // Should return some similarities even with mock data
        assert!(!similarities.is_empty());
    }

    #[tokio::test]
    async fn test_ingredient_suggestions() {
        let service = RecipeSearchService::new();
        let suggestions = service.suggest_ingredients("tom", 5).await.unwrap();
        
        assert!(suggestions.iter().any(|s| s.text == "tomato"));
    }

    #[tokio::test]
    async fn test_typo_correction() {
        let service = RecipeSearchService::new();
        let suggestions = service.suggest_typo_corrections("chickn", 5).await.unwrap();
        
        assert!(suggestions.iter().any(|s| s.text == "chicken"));
    }

    #[tokio::test]
    async fn test_levenshtein_distance() {
        let service = RecipeSearchService::new();
        assert_eq!(service.levenshtein_distance("cat", "bat"), 1);
        assert_eq!(service.levenshtein_distance("chicken", "chickn"), 1);
        assert_eq!(service.levenshtein_distance("same", "same"), 0);
    }
}