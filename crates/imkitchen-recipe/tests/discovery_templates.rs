use askama::Template;
use chrono::Utc;
use imkitchen_recipe::domain::discovery::{DiscoveryFilters, SortingCriteria};
use imkitchen_recipe::projections::discovery::*;
use imkitchen_shared::types::{DietaryRestriction, Difficulty, MealType};
use uuid::Uuid;

// Template for recipe browse view with filtering and pagination
#[derive(Template)]
#[template(
    source = r#"
<div class="recipe-browse-container">
    <div class="filters-section">
        <!-- Applied Filters -->
        {% if let Some(rating) = applied_filters.rating_threshold %}
        <div class="filter-tag">Rating: {{ rating }}+</div>
        {% endif %}
        
        {% if applied_filters.difficulty_levels.len() > 0 %}
        <div class="filter-tag">
            Difficulty: 
            {% for difficulty in applied_filters.difficulty_levels %}
            {{ difficulty }}{% if !loop.last %}, {% endif %}
            {% endfor %}
        </div>
        {% endif %}
        
        {% if let Some(max_prep) = applied_filters.max_prep_time %}
        <div class="filter-tag">Max Prep: {{ max_prep }} min</div>
        {% endif %}
        
        {% if applied_filters.dietary_restrictions.len() > 0 %}
        <div class="filter-tag">
            Dietary: 
            {% for restriction in applied_filters.dietary_restrictions %}
            {{ restriction }}{% if !loop.last %}, {% endif %}
            {% endfor %}
        </div>
        {% endif %}
        
        {% if applied_filters.meal_types.len() > 0 %}
        <div class="filter-tag">
            Meals: 
            {% for meal_type in applied_filters.meal_types %}
            {{ meal_type }}{% if !loop.last %}, {% endif %}
            {% endfor %}
        </div>
        {% endif %}
    </div>
    
    <!-- Sort Order Display -->
    <div class="sort-section">
        <span>Sorted by: {{ sort_order }}</span>
    </div>
    
    <!-- Recipe Cards Grid -->
    <div class="recipe-grid">
        {% for recipe in recipes %}
        <div class="recipe-card" data-recipe-id="{{ recipe.recipe_id }}">
            <h3 class="recipe-title">{{ recipe.title }}</h3>
            <div class="recipe-timing">
                <span class="prep-time">Prep: {{ recipe.prep_time_minutes }}m</span>
                <span class="cook-time">Cook: {{ recipe.cook_time_minutes }}m</span>
                <span class="total-time">Total: {{ recipe.total_time_minutes() }}m</span>
            </div>
            <div class="recipe-meta">
                <span class="difficulty {{ recipe.difficulty_color() }}">{{ recipe.difficulty }}</span>
                <span class="rating">{{ recipe.rating_average }}/5 ({{ recipe.rating_count }})</span>
                {% if recipe.is_highly_rated() %}
                <span class="highly-rated-badge">⭐ Highly Rated</span>
                {% endif %}
            </div>
            {% if let Some(image_url) = &recipe.image_url %}
            <img src="{{ image_url }}" alt="{{ recipe.title }}" class="recipe-image">
            {% endif %}
            <div class="recipe-tags">
                {% for tag in recipe.tags %}
                <span class="tag">{{ tag }}</span>
                {% endfor %}
            </div>
        </div>
        {% endfor %}
    </div>
    
    <!-- Pagination -->
    <div class="pagination">
        <span>Page {{ page }} of {{ (total_count + page_size - 1) / page_size }}</span>
        <span>{{ total_count }} recipes found</span>
        {% if has_more %}
        <button class="load-more">Load More</button>
        {% endif %}
    </div>
</div>
"#,
    ext = "html"
)]
pub struct RecipeBrowseTemplate {
    pub recipes: Vec<RecipeCard>,
    pub total_count: u32,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
    pub applied_filters: DiscoveryFilters,
    pub sort_order: SortingCriteria,
}

// Template for search results with suggestions
#[derive(Template)]
#[template(
    source = r#"
<div class="search-results-container">
    <div class="search-header">
        <h2>Search Results for "{{ query_text }}"</h2>
        <span class="search-stats">{{ total_results }} results found in {{ search_time_ms }}ms</span>
    </div>
    
    <!-- Search Suggestions -->
    {% if suggestions.len() > 0 %}
    <div class="search-suggestions">
        <h3>Suggestions:</h3>
        {% for suggestion in suggestions %}
        <button class="suggestion-button" data-suggestion="{{ suggestion.suggestion_text }}">
            <span class="suggestion-text">{{ suggestion.suggestion_text }}</span>
            <span class="suggestion-type">{{ suggestion.suggestion_type }}</span>
            <span class="suggestion-frequency">({{ suggestion.frequency }})</span>
        </button>
        {% endfor %}
    </div>
    {% endif %}
    
    <!-- Search Results -->
    <div class="search-results">
        {% for result in results %}
        <div class="search-result" data-recipe-id="{{ result.recipe_id }}">
            <h4 class="result-title">{{ result.title }}</h4>
            <p class="result-snippet">{{ result.snippet }}</p>
            <div class="result-meta">
                <span class="relevance-score">Relevance: {{ result.relevance_score }}</span>
                <span class="match-type">Match: {{ result.match_type }}</span>
            </div>
        </div>
        {% endfor %}
    </div>
    
    <!-- Pagination for search results -->
    {% if total_results > page_size %}
    <div class="search-pagination">
        <span>Page {{ page }} of {{ (total_results + page_size - 1) / page_size }}</span>
    </div>
    {% endif %}
</div>
"#,
    ext = "html"
)]
pub struct SearchResultsTemplate {
    pub results: Vec<SearchResult>,
    pub query_text: String,
    pub total_results: u32,
    pub page: u32,
    pub page_size: u32,
    pub suggestions: Vec<SearchSuggestion>,
    pub search_time_ms: u64,
}

// Template for trending recipes
#[derive(Template)]
#[template(
    source = r#"
<div class="trending-recipes-container">
    <div class="trending-header">
        <h2>Trending Recipes ({{ time_window }})</h2>
        <span class="calculated-at">Updated: {{ calculated_at.format("%Y-%m-%d %H:%M UTC") }}</span>
    </div>
    
    <div class="trending-recipes">
        {% for recipe in trending_recipes %}
        <div class="trending-recipe" data-recipe-id="{{ recipe.recipe_id }}" data-rank="{{ recipe.trending_rank }}">
            <div class="trending-rank">{{ recipe.trending_rank }}</div>
            <h3 class="trending-title">{{ recipe.title }}</h3>
            <div class="trending-stats">
                <span class="popularity-score">Popularity: {{ recipe.popularity_score }}</span>
                <span class="view-count">Views (24h): {{ recipe.view_count_24h }}</span>
                <span class="rating">Rating: {{ recipe.rating_average }}/5</span>
                <span class="time-weighted">Time Score: {{ recipe.time_weighted_score }}</span>
            </div>
        </div>
        {% endfor %}
    </div>
    
    {% if trending_recipes.len() > 0 %}
    <div class="trending-highlights">
        <div class="top-recipe">
            <h4>Top Trending:</h4>
            <span>{{ trending_recipes[0].title }}</span>
        </div>
    </div>
    {% endif %}
</div>
"#,
    ext = "html"
)]
pub struct TrendingRecipesTemplate {
    pub time_window: String,
    pub trending_recipes: Vec<TrendingRecipe>,
    pub calculated_at: chrono::DateTime<Utc>,
}

// Template for similar recipes
#[derive(Template)]
#[template(
    source = r#"
<div class="similar-recipes-container">
    <div class="similar-header">
        <h2>Similar Recipes</h2>
        <span class="similarity-threshold">Threshold: {{ similarity_threshold }}</span>
        <span class="generated-at">Generated: {{ generated_at.format("%Y-%m-%d %H:%M UTC") }}</span>
    </div>
    
    <div class="similar-recipes">
        {% for recipe in similar_recipes %}
        <div class="similar-recipe" data-recipe-id="{{ recipe.recipe_id }}">
            <h3 class="similar-title">{{ recipe.title }}</h3>
            <div class="similarity-info">
                <span class="similarity-score">{{ recipe.similarity_score }}</span>
                <span class="similarity-badge">{{ recipe.similarity_badge() }}</span>
                {% if recipe.is_highly_similar() %}
                <span class="highly-similar">🔥 Highly Similar</span>
                {% endif %}
            </div>
            <div class="similarity-breakdown">
                <span class="ingredient-overlap">Ingredients: {{ recipe.ingredient_overlap }}</span>
                <span class="technique-similarity">Technique: {{ recipe.technique_similarity }}</span>
            </div>
            <div class="similarity-reasons">
                <h4>Why it's similar:</h4>
                {% for reason in recipe.similarity_reasons %}
                <p class="reason">{{ reason }}</p>
                {% endfor %}
            </div>
        </div>
        {% endfor %}
    </div>
</div>
"#,
    ext = "html"
)]
pub struct SimilarRecipesTemplate {
    pub original_recipe_id: Uuid,
    pub similar_recipes: Vec<SimilarRecipe>,
    pub similarity_threshold: f64,
    pub generated_at: chrono::DateTime<Utc>,
}

// Template for user preferences view
#[derive(Template)]
#[template(
    source = r#"
<div class="user-preferences-container">
    <div class="preferences-header">
        <h2>Your Recipe Preferences</h2>
        <span class="updated-at">Last updated: {{ updated_at.format("%Y-%m-%d %H:%M UTC") }}</span>
    </div>
    
    <div class="preferences-content">
        <div class="difficulty-preferences">
            <h3>Preferred Difficulty:</h3>
            <span class="difficulty-text">{{ difficulty_preference_text() }}</span>
        </div>
        
        {% if preferred_meal_types.len() > 0 %}
        <div class="meal-type-preferences">
            <h3>Preferred Meal Types:</h3>
            {% for meal_type in preferred_meal_types %}
            <span class="meal-type-tag">{{ meal_type }}</span>
            {% endfor %}
        </div>
        {% endif %}
        
        {% if preferred_dietary_restrictions.len() > 0 %}
        <div class="dietary-preferences">
            <h3>Dietary Restrictions:</h3>
            {% for restriction in preferred_dietary_restrictions %}
            <span class="dietary-tag">{{ restriction }}</span>
            {% endfor %}
        </div>
        {% endif %}
        
        <div class="time-preferences">
            <h3>Time Preferences:</h3>
            <span class="avg-prep-time">Average preferred prep time: {{ avg_prep_time_preference }} minutes</span>
            {% if prefers_quick_recipes() %}
            <span class="quick-recipe-badge">⚡ Prefers Quick Recipes</span>
            {% endif %}
        </div>
        
        {% if favorite_ingredients.len() > 0 %}
        <div class="ingredient-preferences">
            <h3>Favorite Ingredients:</h3>
            {% for ingredient in favorite_ingredients %}
            <span class="ingredient-tag favorite">{{ ingredient }}</span>
            {% endfor %}
        </div>
        {% endif %}
        
        {% if disliked_ingredients.len() > 0 %}
        <div class="disliked-ingredients">
            <h3>Disliked Ingredients:</h3>
            {% for ingredient in disliked_ingredients %}
            <span class="ingredient-tag disliked">{{ ingredient }}</span>
            {% endfor %}
        </div>
        {% endif %}
    </div>
</div>
"#,
    ext = "html"
)]
pub struct UserPreferencesTemplate {
    pub user_id: Uuid,
    pub preferred_difficulties: Vec<Difficulty>,
    pub preferred_meal_types: Vec<String>,
    pub preferred_dietary_restrictions: Vec<String>,
    pub avg_prep_time_preference: u32,
    pub favorite_ingredients: Vec<String>,
    pub disliked_ingredients: Vec<String>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl UserPreferencesTemplate {
    pub fn prefers_quick_recipes(&self) -> bool {
        self.avg_prep_time_preference <= 30
    }

    pub fn difficulty_preference_text(&self) -> String {
        if self.preferred_difficulties.is_empty() {
            "No preference".to_string()
        } else {
            format!("{:?}", self.preferred_difficulties)
        }
    }
}

// Template for discovery analytics
#[derive(Template)]
#[template(
    source = r#"
<div class="discovery-analytics-container">
    <div class="analytics-header">
        <h2>Discovery Analytics ({{ time_period }})</h2>
        <span class="generated-at">Generated: {{ generated_at.format("%Y-%m-%d %H:%M UTC") }}</span>
    </div>
    
    <div class="analytics-metrics">
        <div class="metric-card">
            <h3>Total Searches</h3>
            <span class="metric-value">{{ total_searches }}</span>
        </div>
        
        <div class="metric-card">
            <h3>Filter Applications</h3>
            <span class="metric-value">{{ total_filter_applications }}</span>
        </div>
        
        <div class="metric-card">
            <h3>Recipe Views</h3>
            <span class="metric-value">{{ total_recipe_views }}</span>
        </div>
        
        <div class="metric-card">
            <h3>Conversion Rate</h3>
            <span class="metric-value">{{ conversion_rate }}%</span>
        </div>
        
        <div class="metric-card">
            <h3>Search Success Rate</h3>
            <span class="metric-value">{{ search_success_rate() }}%</span>
        </div>
    </div>
    
    {% if popular_search_terms.len() > 0 %}
    <div class="popular-searches">
        <h3>Popular Search Terms:</h3>
        {% for term in popular_search_terms %}
        <span class="search-term">{{ term }}</span>
        {% endfor %}
    </div>
    {% endif %}
    
    {% if popular_filters.len() > 0 %}
    <div class="popular-filters">
        <h3>Popular Filters:</h3>
        {% for filter in popular_filters %}
        <span class="filter-term">{{ filter }}</span>
        {% endfor %}
    </div>
    {% endif %}
</div>
"#,
    ext = "html"
)]
pub struct DiscoveryAnalyticsTemplate {
    pub time_period: String,
    pub total_searches: u32,
    pub total_filter_applications: u32,
    pub total_recipe_views: u32,
    pub popular_search_terms: Vec<String>,
    pub popular_filters: Vec<String>,
    pub conversion_rate: f64,
    pub generated_at: chrono::DateTime<Utc>,
}

impl DiscoveryAnalyticsTemplate {
    pub fn search_success_rate(&self) -> f64 {
        if self.total_searches == 0 {
            0.0
        } else {
            (self.total_recipe_views as f64) / (self.total_searches as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_browse_template() {
        let recipe_card = RecipeCard {
            recipe_id: Uuid::new_v4(),
            title: "Test Pasta Recipe".to_string(),
            prep_time_minutes: 15,
            cook_time_minutes: 20,
            difficulty: Difficulty::Easy,
            rating_average: 4.5,
            rating_count: 25,
            image_url: Some("pasta.jpg".to_string()),
            created_by: Uuid::new_v4(),
            tags: vec!["pasta".to_string(), "italian".to_string()],
        };

        let template = RecipeBrowseTemplate {
            recipes: vec![recipe_card],
            total_count: 1,
            page: 1,
            page_size: 20,
            has_more: false,
            applied_filters: DiscoveryFilters {
                rating_threshold: Some(4.0),
                difficulty_levels: vec![Difficulty::Easy],
                max_prep_time: Some(30),
                dietary_restrictions: vec![DietaryRestriction::Vegetarian],
                meal_types: vec![MealType::Dinner],
            },
            sort_order: SortingCriteria::HighestRated,
        };

        let rendered = template
            .render()
            .expect("Template should render successfully");

        assert!(rendered.contains("Test Pasta Recipe"));
        assert!(rendered.contains("Rating: 4+"));
        assert!(rendered.contains("Difficulty:") && rendered.contains("Easy"));
        assert!(rendered.contains("Total: 35m"));
        assert!(rendered.contains("4.5/5 (25)"));
        assert!(rendered.contains("⭐ Highly Rated"));
        assert!(rendered.contains("Sorted by: Highest Rated"));
    }

    #[test]
    fn test_search_results_template() {
        let search_result = SearchResult {
            recipe_id: Uuid::new_v4(),
            title: "Delicious Pasta".to_string(),
            snippet: "A wonderful pasta recipe with fresh ingredients".to_string(),
            relevance_score: 0.95,
            match_type: "title".to_string(),
        };

        let suggestion = SearchSuggestion {
            suggestion_text: "pasta".to_string(),
            suggestion_type: "ingredient".to_string(),
            frequency: 150,
        };

        let template = SearchResultsTemplate {
            results: vec![search_result],
            query_text: "pasta".to_string(),
            total_results: 1,
            page: 1,
            page_size: 20,
            suggestions: vec![suggestion],
            search_time_ms: 50,
        };

        let rendered = template
            .render()
            .expect("Template should render successfully");

        assert!(rendered.contains("Search Results for \"pasta\""));
        assert!(rendered.contains("1 results found in 50ms"));
        assert!(rendered.contains("Delicious Pasta"));
        assert!(rendered.contains("Relevance: 0.95"));
        assert!(rendered.contains("Match: title"));
        assert!(rendered.contains("Suggestions:"));
        assert!(rendered.contains("pasta"));
        assert!(rendered.contains("ingredient"));
        assert!(rendered.contains("(150)"));
    }

    #[test]
    fn test_trending_recipes_template() {
        let trending_recipe = TrendingRecipe {
            recipe_id: Uuid::new_v4(),
            title: "Viral Pasta Recipe".to_string(),
            popularity_score: 95.5,
            trending_rank: 1,
            view_count_24h: 1500,
            rating_average: 4.8,
            time_weighted_score: 98.2,
        };

        let template = TrendingRecipesTemplate {
            time_window: "24h".to_string(),
            trending_recipes: vec![trending_recipe],
            calculated_at: Utc::now(),
        };

        let rendered = template
            .render()
            .expect("Template should render successfully");

        assert!(rendered.contains("Trending Recipes (24h)"));
        assert!(rendered.contains("Viral Pasta Recipe"));
        assert!(rendered.contains("Popularity: 95.5"));
        assert!(rendered.contains("Views (24h): 1500"));
        assert!(rendered.contains("Rating: 4.8/5"));
        assert!(rendered.contains("Top Trending:"));
    }

    #[test]
    fn test_similar_recipes_template() {
        let similar_recipe = SimilarRecipe {
            recipe_id: Uuid::new_v4(),
            title: "Similar Pasta Dish".to_string(),
            similarity_score: 0.85,
            similarity_reasons: vec![
                "Common ingredients: pasta, cheese".to_string(),
                "Similar cooking technique: boiling".to_string(),
            ],
            ingredient_overlap: 0.75,
            technique_similarity: 0.90,
        };

        let template = SimilarRecipesTemplate {
            original_recipe_id: Uuid::new_v4(),
            similar_recipes: vec![similar_recipe],
            similarity_threshold: 0.7,
            generated_at: Utc::now(),
        };

        let rendered = template
            .render()
            .expect("Template should render successfully");

        assert!(rendered.contains("Similar Recipes"));
        assert!(rendered.contains("Similar Pasta Dish"));
        assert!(rendered.contains("0.85"));
        assert!(rendered.contains("Similar"));
        assert!(rendered.contains("🔥 Highly Similar"));
        assert!(rendered.contains("Common ingredients: pasta, cheese"));
        assert!(rendered.contains("Ingredients: 0.75"));
        assert!(rendered.contains("Technique: 0.9"));
    }

    #[test]
    fn test_user_preferences_template() {
        let template = UserPreferencesTemplate {
            user_id: Uuid::new_v4(),
            preferred_difficulties: vec![Difficulty::Easy, Difficulty::Medium],
            preferred_meal_types: vec!["Dinner".to_string(), "Lunch".to_string()],
            preferred_dietary_restrictions: vec!["Vegetarian".to_string()],
            avg_prep_time_preference: 25,
            favorite_ingredients: vec!["pasta".to_string(), "cheese".to_string()],
            disliked_ingredients: vec!["olives".to_string()],
            updated_at: Utc::now(),
        };

        let rendered = template
            .render()
            .expect("Template should render successfully");

        assert!(rendered.contains("Your Recipe Preferences"));
        assert!(rendered.contains("⚡ Prefers Quick Recipes"));
        assert!(rendered.contains("Average preferred prep time: 25 minutes"));
        assert!(rendered.contains("Dinner"));
        assert!(rendered.contains("Vegetarian"));
        assert!(rendered.contains("pasta"));
        assert!(rendered.contains("olives"));
    }

    #[test]
    fn test_discovery_analytics_template() {
        let template = DiscoveryAnalyticsTemplate {
            time_period: "last 7 days".to_string(),
            total_searches: 150,
            total_filter_applications: 89,
            total_recipe_views: 320,
            popular_search_terms: vec!["pasta".to_string(), "chicken".to_string()],
            popular_filters: vec!["vegetarian".to_string(), "quick".to_string()],
            conversion_rate: 75.5,
            generated_at: Utc::now(),
        };

        let rendered = template
            .render()
            .expect("Template should render successfully");

        assert!(rendered.contains("Discovery Analytics (last 7 days)"));
        assert!(rendered.contains("Total Searches"));
        assert!(rendered.contains("150"));
        assert!(rendered.contains("320"));
        assert!(rendered.contains("75.5%"));
        assert!(rendered.contains("Popular Search Terms:"));
        assert!(rendered.contains("pasta"));
        assert!(rendered.contains("chicken"));
        assert!(rendered.contains("Popular Filters:"));
        assert!(rendered.contains("vegetarian"));
    }

    #[test]
    fn test_recipe_card_methods() {
        let recipe = RecipeCard {
            recipe_id: Uuid::new_v4(),
            title: "Test Recipe".to_string(),
            prep_time_minutes: 15,
            cook_time_minutes: 20,
            difficulty: Difficulty::Easy,
            rating_average: 4.5,
            rating_count: 25,
            image_url: None,
            created_by: Uuid::new_v4(),
            tags: vec![],
        };

        assert_eq!(recipe.total_time_minutes(), 35);
        assert!(recipe.is_highly_rated());
        assert_eq!(recipe.difficulty_color(), "green");
    }

    #[test]
    fn test_similar_recipe_methods() {
        let recipe = SimilarRecipe {
            recipe_id: Uuid::new_v4(),
            title: "Test Recipe".to_string(),
            similarity_score: 0.85,
            similarity_reasons: vec![],
            ingredient_overlap: 0.75,
            technique_similarity: 0.90,
        };

        assert!(recipe.is_highly_similar());
        assert_eq!(recipe.similarity_badge(), "Similar");

        let very_similar = SimilarRecipe {
            recipe_id: Uuid::new_v4(),
            title: "Very Similar Recipe".to_string(),
            similarity_score: 0.95,
            similarity_reasons: vec![],
            ingredient_overlap: 0.95,
            technique_similarity: 0.95,
        };

        assert_eq!(very_similar.similarity_badge(), "Very Similar");
    }
}
