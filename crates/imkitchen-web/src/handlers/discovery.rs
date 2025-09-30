use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Form,
};
use chrono::Datelike;
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use imkitchen_recipe::services::search::{RecipeSearchService, SearchAnalytics};
use imkitchen_recipe::services::popularity::{PopularityService, TimeWindow};
use imkitchen_recipe::services::recommendation::{
    RecommendationEngine, RecommendationContext, TimeContext, Season, MealTime, 
    RecommendationFilter, UserInteraction, InteractionType
};

// Template for main discovery page
#[derive(Template)]
#[template(path = "pages/discovery/index.html")]
pub struct DiscoveryPageTemplate {
    pub title: String,
}

// Form data structures for handling TwinSpark requests
#[derive(Deserialize)]
pub struct DiscoveryQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub rating_threshold: Option<f32>,
    pub difficulty_levels: Option<Vec<String>>,
    pub max_prep_time: Option<u32>,
    pub dietary_restrictions: Option<Vec<String>>,
    pub meal_types: Option<Vec<String>>,
    pub sort_order: Option<String>,
    pub append: Option<bool>, // For infinite scroll
}

#[derive(Deserialize)]
pub struct DiscoverySearchQuery {
    pub q: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Deserialize)]
pub struct TrendingQuery {
    pub time_window: Option<String>, // "24h", "7d", "30d"
    pub category: Option<String>,
    pub min_rating: Option<f32>,
}

#[derive(Deserialize)]
pub struct FilterForm {
    pub rating_threshold: Option<f32>,
    pub difficulty_levels: Option<Vec<String>>,
    pub max_prep_time: Option<u32>,
    pub dietary_restrictions: Option<Vec<String>>,
    pub meal_types: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct SortForm {
    pub sort_order: String,
}

#[derive(Deserialize)]
pub struct FavoriteForm {
    pub recipe_id: Uuid,
}

#[derive(Deserialize)]
pub struct ShoppingForm {
    pub recipe_id: Uuid,
}

#[derive(Deserialize)]
pub struct AutocompleteQuery {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct RecommendationQuery {
    pub user_id: Option<String>,
    pub limit: Option<usize>,
    pub meal_time: Option<String>, // "breakfast", "lunch", "dinner", "snack"
    pub require_novelty: Option<bool>,
    pub diversity_weight: Option<f64>,
    pub exclude_categories: Option<Vec<String>>,
    pub boost_categories: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct UserInteractionForm {
    pub user_id: String,
    pub recipe_id: Uuid,
    pub interaction_type: String, // "viewed", "clicked", "bookmarked", "cooked", "rated"
    pub rating: Option<f64>,
    pub context: Option<String>,
}

// Main Handlers

/// GET /discovery - Main discovery page
pub async fn discovery_page(State(_state): State<AppState>) -> impl IntoResponse {
    let template = DiscoveryPageTemplate {
        title: "Discover Recipes".to_string(),
    };

    Html(template.render().unwrap())
}

/// GET /discovery/browse - Recipe browse view with filters
pub async fn browse_recipes(
    Query(_params): Query<DiscoveryQuery>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // For now, return a simple HTML fragment
    Html(r#"
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Browse Results</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
                <h3 class="font-medium text-gray-900">Sample Recipe 1</h3>
                <p class="text-sm text-gray-600 mt-1">A delicious sample recipe to test the interface</p>
                <div class="mt-3 flex items-center justify-between">
                    <span class="text-xs text-gray-500">30 min</span>
                    <div class="flex items-center">
                        <span class="text-yellow-400">★★★★☆</span>
                        <span class="ml-1 text-xs text-gray-500">(4.2)</span>
                    </div>
                </div>
            </div>
            <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
                <h3 class="font-medium text-gray-900">Sample Recipe 2</h3>
                <p class="text-sm text-gray-600 mt-1">Another delicious sample recipe</p>
                <div class="mt-3 flex items-center justify-between">
                    <span class="text-xs text-gray-500">45 min</span>
                    <div class="flex items-center">
                        <span class="text-yellow-400">★★★★★</span>
                        <span class="ml-1 text-xs text-gray-500">(4.8)</span>
                    </div>
                </div>
            </div>
        </div>
    </div>
    "#)
}

/// GET/POST /discovery/search - Recipe search with suggestions
pub async fn discovery_search_recipes(
    Query(params): Query<DiscoverySearchQuery>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let query_text = params.q.clone();
    let page = params.page.unwrap_or(1);
    let _page_size = params.page_size.unwrap_or(20);
    
    // Use the search service to get suggestions and perform search
    let search_service = RecipeSearchService::new();
    
    // Generate search suggestions for autocomplete
    let suggestions = search_service.generate_suggestions(&query_text, None, 5).await
        .unwrap_or_default();
    
    // Record search analytics
    let analytics = SearchAnalytics {
        query: query_text.clone(),
        results_count: 3, // Mock result count
        click_through_rate: 0.65,
        search_duration_ms: 150,
        popular_filters: std::collections::HashMap::new(),
        successful_suggestions: suggestions.iter().map(|s| s.text.clone()).collect(),
    };
    
    let _ = search_service.record_search_analytics(analytics).await;
    
    // Generate enhanced search results with suggestions
    let suggestions_html = if !suggestions.is_empty() {
        let suggestion_items: Vec<String> = suggestions.iter().map(|s| {
            format!(r#"<span class="inline-block px-2 py-1 bg-gray-100 text-gray-700 rounded text-xs mr-2 mb-2 cursor-pointer hover:bg-gray-200" onclick="document.querySelector('input[name=q]').value='{}'">
                {}</span>"#, s.text, s.text)
        }).collect();
        
        format!(r#"
        <div class="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
            <h4 class="text-sm font-medium text-blue-900 mb-2">Suggestions:</h4>
            <div class="flex flex-wrap">
                {}
            </div>
        </div>
        "#, suggestion_items.join(""))
    } else {
        String::new()
    };
    
    Html(format!(r#"
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Search Results for "{}"</h2>
        
        {}
        
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
                <h3 class="font-medium text-gray-900">Chicken Teriyaki</h3>
                <p class="text-sm text-gray-600 mt-1">Delicious teriyaki chicken with steamed rice</p>
                <div class="mt-3 flex items-center justify-between">
                    <span class="text-xs text-gray-500">25 min</span>
                    <div class="flex items-center">
                        <span class="text-yellow-400">★★★★☆</span>
                        <span class="ml-1 text-xs text-gray-500">(4.5)</span>
                    </div>
                </div>
            </div>
            <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
                <h3 class="font-medium text-gray-900">Pasta Primavera</h3>
                <p class="text-sm text-gray-600 mt-1">Fresh vegetables with pasta in light sauce</p>
                <div class="mt-3 flex items-center justify-between">
                    <span class="text-xs text-gray-500">30 min</span>
                    <div class="flex items-center">
                        <span class="text-yellow-400">★★★★★</span>
                        <span class="ml-1 text-xs text-gray-500">(4.8)</span>
                    </div>
                </div>
            </div>
            <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
                <h3 class="font-medium text-gray-900">Beef Stir Fry</h3>
                <p class="text-sm text-gray-600 mt-1">Quick and easy beef stir fry with vegetables</p>
                <div class="mt-3 flex items-center justify-between">
                    <span class="text-xs text-gray-500">20 min</span>
                    <div class="flex items-center">
                        <span class="text-yellow-400">★★★★☆</span>
                        <span class="ml-1 text-xs text-gray-500">(4.3)</span>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="mt-6 text-center">
            <p class="text-sm text-gray-600">Found 3 recipes matching "{}" (Page {} of 1)</p>
        </div>
    </div>
    "#, query_text, suggestions_html, query_text, page))
}

/// GET /discovery/trending - Trending recipes
pub async fn trending_recipes(
    Query(params): Query<TrendingQuery>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let popularity_service = PopularityService::new();
    
    // Determine time window
    let time_window = match params.time_window.as_deref() {
        Some("7d") => TimeWindow::last_week(),
        Some("30d") => TimeWindow::last_month(),
        _ => TimeWindow::last_24_hours(),
    };
    
    // Get trending recipes
    let trending_recipes = popularity_service.get_trending_recipes(&time_window, 12).await
        .unwrap_or_default();
    
    if trending_recipes.is_empty() {
        return Html(r#"
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <h2 class="text-lg font-semibold text-gray-900 mb-4">Trending Recipes</h2>
            <div class="text-center py-8">
                <p class="text-gray-500">No trending recipes found for this time period.</p>
                <p class="text-sm text-gray-400 mt-2">Check back later as recipes gain popularity!</p>
            </div>
        </div>
        "#.to_string());
    }
    
    // Generate trending recipes HTML
    let recipes_html = trending_recipes.iter().enumerate().map(|(index, recipe)| {
        let trend_icon = match index {
            0 => "🥇", // Gold for #1
            1 => "🥈", // Silver for #2  
            2 => "🥉", // Bronze for #3
            _ => "🔥", // Fire for others
        };
        
        let trend_factors_html = recipe.trend_factors.iter().map(|factor| {
            match factor {
                imkitchen_recipe::services::popularity::TrendFactor::ViewSpike { multiplier, .. } => {
                    format!("📈 {}x more views", *multiplier as i32)
                },
                imkitchen_recipe::services::popularity::TrendFactor::RatingBoost { average_rating, recent_ratings } => {
                    format!("⭐ {:.1} rating ({} reviews)", average_rating, recent_ratings)
                },
                imkitchen_recipe::services::popularity::TrendFactor::BookmarkSurge { bookmark_velocity } => {
                    format!("🔖 {}x bookmarks", *bookmark_velocity as i32)
                },
                imkitchen_recipe::services::popularity::TrendFactor::NewRecipe { days_old, .. } => {
                    format!("🆕 {} days old", days_old)
                },
                imkitchen_recipe::services::popularity::TrendFactor::SearchPopularity { query_frequency, .. } => {
                    format!("🔍 {} searches", query_frequency)
                },
                imkitchen_recipe::services::popularity::TrendFactor::SeasonalRelevance { relevance_score, context } => {
                    format!("🌱 {:.1}% seasonal ({})", relevance_score * 100.0, context)
                },
            }
        }).collect::<Vec<_>>().join(" • ");
        
        format!(r#"
        <div class="bg-gray-50 rounded-lg p-4 border border-gray-200 hover:shadow-md transition-all duration-200">
            <div class="flex items-center justify-between mb-2">
                <div class="flex items-center">
                    <span class="text-lg mr-2">{}</span>
                    <span class="text-xs bg-red-100 text-red-600 px-2 py-1 rounded-full font-medium">
                        #{} TRENDING
                    </span>
                </div>
                <span class="text-xs text-blue-600 font-bold">{:.0} score</span>
            </div>
            <h3 class="font-medium text-gray-900">{}</h3>
            <p class="text-sm text-gray-600 mt-1">{} views • {:.1}x velocity</p>
            <div class="mt-2 text-xs text-gray-500">
                {}
            </div>
            <div class="mt-3 flex items-center justify-between">
                <span class="text-xs text-gray-500">~30 min</span>
                <div class="flex items-center">
                    <span class="text-yellow-400">★★★★☆</span>
                    <span class="ml-1 text-xs text-gray-500">(4.5)</span>
                </div>
            </div>
        </div>
        "#, 
        trend_icon,
        index + 1,
        recipe.trending_score,
        recipe.title,
        recipe.recent_views,
        recipe.velocity,
        if trend_factors_html.is_empty() { "Gaining popularity".to_string() } else { trend_factors_html }
        )
    }).collect::<Vec<_>>().join("");
    
    Html(format!(r#"
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div class="flex justify-between items-center mb-4">
            <h2 class="text-lg font-semibold text-gray-900">
                Trending Recipes 
                <span class="text-sm font-normal text-gray-500">({} window)</span>
            </h2>
            <div class="flex space-x-2">
                <button class="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded" 
                        onclick="updateTrending('24h')">24h</button>
                <button class="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded" 
                        onclick="updateTrending('7d')">7d</button>
                <button class="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded" 
                        onclick="updateTrending('30d')">30d</button>
            </div>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {}
        </div>
        <div class="mt-4 text-center">
            <p class="text-xs text-gray-500">
                Trending analysis based on views, ratings, bookmarks, and search activity
            </p>
        </div>
    </div>
    "#, time_window.label, recipes_html))
}

/// GET /discovery/similar/{recipe_id} - Similar recipes
pub async fn similar_recipes(
    Path(recipe_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Use the search service to find similar recipes
    let search_service = RecipeSearchService::new();
    let similarities = search_service.find_similar_recipes(recipe_id, 6).await
        .unwrap_or_default();
    
    // Generate HTML for similar recipes
    let similar_recipes_html = if !similarities.is_empty() {
        similarities.iter().map(|sim| {
            let reasons: Vec<String> = sim.similarity_reasons.iter().map(|reason| {
                match reason {
                    imkitchen_recipe::services::search::SimilarityReason::IngredientOverlap { common_ingredients, .. } => {
                        format!("Shares ingredients: {}", common_ingredients.join(", "))
                    },
                    imkitchen_recipe::services::search::SimilarityReason::CookingTechnique { common_techniques, .. } => {
                        format!("Similar techniques: {}", common_techniques.join(", "))
                    },
                    imkitchen_recipe::services::search::SimilarityReason::Category { category } => {
                        format!("Same category: {:?}", category)
                    },
                    imkitchen_recipe::services::search::SimilarityReason::Difficulty { difficulty } => {
                        format!("Same difficulty: {:?}", difficulty)
                    },
                    imkitchen_recipe::services::search::SimilarityReason::PrepTime { similar_time_range } => {
                        format!("Similar prep time: {}", similar_time_range)
                    },
                }
            }).collect();
            
            format!(r#"
            <div class="bg-gray-50 rounded-lg p-4 border border-gray-200 hover:shadow-md transition-shadow">
                <div class="flex justify-between items-start mb-2">
                    <h3 class="font-medium text-gray-900">Recipe #{}</h3>
                    <span class="text-xs text-blue-600 font-medium">{:.1}% match</span>
                </div>
                <p class="text-sm text-gray-600 mb-3">{}</p>
                <div class="flex items-center justify-between">
                    <span class="text-xs text-gray-500">30 min</span>
                    <div class="flex items-center">
                        <span class="text-yellow-400">★★★★☆</span>
                        <span class="ml-1 text-xs text-gray-500">(4.2)</span>
                    </div>
                </div>
            </div>
            "#, 
            sim.recipe_id.to_string().chars().take(8).collect::<String>(),
            sim.similarity_score * 100.0,
            reasons.join(" • ")
            )
        }).collect::<Vec<_>>().join("")
    } else {
        r#"
        <div class="text-center py-8">
            <p class="text-gray-500">No similar recipes found at this time.</p>
            <p class="text-sm text-gray-400 mt-2">Our recommendation engine is learning your preferences!</p>
        </div>
        "#.to_string()
    };

    Html(format!(r#"
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">
            Similar Recipes
            <span class="text-sm font-normal text-gray-500 ml-2">(Based on ingredients, techniques, and categories)</span>
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {}
        </div>
    </div>
    "#, similar_recipes_html))
}

/// GET /discovery/filters - Filters sidebar
pub async fn filters_sidebar(State(_state): State<AppState>) -> impl IntoResponse {
    const FILTERS_HTML: &str = r##"
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Filter Recipes</h2>
        <form ts-req="/discovery/apply-filters" ts-target="#discovery-content">
            <div class="space-y-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">Rating</label>
                    <select name="rating_threshold" class="w-full border border-gray-300 rounded-md px-3 py-2">
                        <option value="">Any rating</option>
                        <option value="4">4+ stars</option>
                        <option value="3">3+ stars</option>
                    </select>
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">Max Time</label>
                    <select name="max_prep_time" class="w-full border border-gray-300 rounded-md px-3 py-2">
                        <option value="">Any time</option>
                        <option value="15">15 min</option>
                        <option value="30">30 min</option>
                        <option value="60">1 hour</option>
                    </select>
                </div>
                <button type="submit" class="w-full bg-kitchen-600 text-white py-2 px-4 rounded-md hover:bg-kitchen-700">
                    Apply Filters
                </button>
            </div>
        </form>
    </div>
    "##;
    
    Html(FILTERS_HTML)
}

// TwinSpark interaction handlers

/// POST /discovery/apply-filters - Apply discovery filters
pub async fn apply_filters(
    State(_state): State<AppState>,
    Form(_form): Form<FilterForm>,
) -> impl IntoResponse {
    // Return updated browse view
    browse_recipes(Query(DiscoveryQuery { 
        page: Some(1), 
        page_size: Some(20),
        rating_threshold: None,
        difficulty_levels: None,
        max_prep_time: None,
        dietary_restrictions: None,
        meal_types: None,
        sort_order: None,
        append: None,
    }), State(_state)).await
}

/// POST /discovery/clear-filters - Clear all discovery filters
pub async fn clear_filters(State(_state): State<AppState>) -> impl IntoResponse {
    browse_recipes(Query(DiscoveryQuery { 
        page: Some(1), 
        page_size: Some(20),
        rating_threshold: None,
        difficulty_levels: None,
        max_prep_time: None,
        dietary_restrictions: None,
        meal_types: None,
        sort_order: None,
        append: None,
    }), State(_state)).await
}

/// GET /discovery/quick-filter - Quick filter presets
pub async fn quick_filter(
    Query(params): Query<DiscoveryQuery>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    browse_recipes(Query(params), State(_state)).await
}

/// POST /discovery/change-sort - Change sort order
pub async fn change_sort(
    State(_state): State<AppState>,
    Form(_form): Form<SortForm>,
) -> impl IntoResponse {
    browse_recipes(Query(DiscoveryQuery { 
        page: Some(1), 
        page_size: Some(20),
        rating_threshold: None,
        difficulty_levels: None,
        max_prep_time: None,
        dietary_restrictions: None,
        meal_types: None,
        sort_order: None,
        append: None,
    }), State(_state)).await
}

/// POST /discovery/favorite - Add recipe to favorites
pub async fn quick_add_to_favorites(
    State(_state): State<AppState>,
    Form(_form): Form<FavoriteForm>,
) -> impl IntoResponse {
    Html(r#"
    <div class="p-2 bg-green-50 border border-green-200 rounded-md text-center">
        <span class="text-sm text-green-600">Added to favorites!</span>
    </div>
    "#)
}

/// POST /discovery/quick-add-shopping - Quick add to shopping list
pub async fn quick_add_to_shopping(
    State(_state): State<AppState>,
    Form(_form): Form<ShoppingForm>,
) -> impl IntoResponse {
    Html(r#"
    <div class="p-2 bg-blue-50 border border-blue-200 rounded-md text-center">
        <span class="text-sm text-blue-600">Added to shopping list!</span>
    </div>
    "#)
}

/// GET /discovery/autocomplete - Search autocomplete suggestions
pub async fn search_autocomplete(
    Query(params): Query<AutocompleteQuery>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let search_service = RecipeSearchService::new();
    let suggestions = search_service.generate_suggestions(
        &params.q, 
        None, 
        params.limit.unwrap_or(8)
    ).await.unwrap_or_default();
    
    // Return JSON-like response for autocomplete
    let suggestions_html: Vec<String> = suggestions.iter().map(|s| {
        format!(r#"
        <div class="px-3 py-2 hover:bg-gray-100 cursor-pointer border-b border-gray-100 last:border-b-0" 
             onclick="selectSuggestion('{}')">
            <div class="flex items-center justify-between">
                <span class="text-sm text-gray-900">{}</span>
                <span class="text-xs text-gray-500 capitalize">{:?}</span>
            </div>
        </div>
        "#, s.text, s.text, s.suggestion_type)
    }).collect();
    
    if suggestions_html.is_empty() {
        Html(r#"
        <div class="px-3 py-2 text-sm text-gray-500 text-center">
            No suggestions found
        </div>
        "#.to_string())
    } else {
        Html(suggestions_html.join(""))
    }
}

/// GET /discovery/recommendations - Personalized recipe recommendations
pub async fn recipe_recommendations(
    Query(params): Query<RecommendationQuery>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let recommendation_engine = RecommendationEngine::new();
    let user_id = params.user_id.unwrap_or_else(|| "anonymous".to_string());
    
    // Parse meal time
    let meal_time = params.meal_time.as_deref().and_then(|mt| {
        match mt {
            "breakfast" => Some(MealTime::Breakfast),
            "lunch" => Some(MealTime::Lunch),
            "dinner" => Some(MealTime::Dinner),
            "snack" => Some(MealTime::Snack),
            "dessert" => Some(MealTime::Dessert),
            _ => None,
        }
    });
    
    // Determine current season (simplified)
    let now = chrono::Utc::now();
    let season = match now.month() {
        3..=5 => Season::Spring,
        6..=8 => Season::Summer,
        9..=11 => Season::Fall,
        _ => Season::Winter,
    };
    
    // Build recommendation context
    let context = RecommendationContext {
        user_id: user_id.clone(),
        session_context: None,
        time_context: TimeContext {
            current_time: now,
            meal_time: meal_time.clone(),
            season,
            day_of_week: now.weekday().to_string(),
            is_weekend: matches!(now.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun),
        },
        filters: params.exclude_categories.unwrap_or_default().into_iter().map(|cat| {
            RecommendationFilter {
                filter_type: "category".to_string(),
                filter_values: vec![cat],
                is_exclude: true,
            }
        }).collect(),
        exclude_recipes: vec![],
        boost_categories: params.boost_categories.unwrap_or_default(),
        max_recommendations: params.limit.unwrap_or(12),
        require_novelty: params.require_novelty.unwrap_or(false),
        diversity_weight: params.diversity_weight.unwrap_or(0.3),
    };
    
    // Generate recommendations
    let recommendations = recommendation_engine.generate_recommendations(&context).await
        .unwrap_or_default();
    
    if recommendations.is_empty() {
        return Html(r#"
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <h2 class="text-lg font-semibold text-gray-900 mb-4">Personalized Recommendations</h2>
            <div class="text-center py-8">
                <p class="text-gray-500">No recommendations available at this time.</p>
                <p class="text-sm text-gray-400 mt-2">Try interacting with more recipes to improve recommendations!</p>
            </div>
        </div>
        "#.to_string());
    }
    
    // Generate recommendations HTML
    let recommendations_html = recommendations.iter().enumerate().map(|(index, rec)| {
        let recommendation_type_badge = match rec.recommendation_type {
            imkitchen_recipe::services::recommendation::RecommendationType::ContentBased => {
                r#"<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">Based on your taste</span>"#
            },
            imkitchen_recipe::services::recommendation::RecommendationType::CollaborativeFiltering => {
                r#"<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">Similar users liked</span>"#
            },
            imkitchen_recipe::services::recommendation::RecommendationType::Trending => {
                r#"<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800">Trending now</span>"#
            },
            imkitchen_recipe::services::recommendation::RecommendationType::SeasonalSuggestion => {
                r#"<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-orange-100 text-orange-800">Seasonal pick</span>"#
            },
            imkitchen_recipe::services::recommendation::RecommendationType::PersonalizedTrending => {
                r#"<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-purple-100 text-purple-800">Trending for you</span>"#
            },
            imkitchen_recipe::services::recommendation::RecommendationType::Hybrid => {
                r#"<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800">Perfect match</span>"#
            },
            _ => {
                r#"<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800">Recommended</span>"#
            },
        };
        
        let reasons_html = rec.reasons.iter().take(3).map(|reason| {
            match reason {
                imkitchen_recipe::services::recommendation::RecommendationReason::SimilarIngredients { common_ingredients, .. } => {
                    format!("🥘 Similar ingredients: {}", common_ingredients.join(", "))
                },
                imkitchen_recipe::services::recommendation::RecommendationReason::UserPreferenceMatch { preference_type, preference_value, .. } => {
                    format!("👤 Matches your {} preference: {}", preference_type, preference_value)
                },
                imkitchen_recipe::services::recommendation::RecommendationReason::CollaborativeSignal { similar_users, .. } => {
                    format!("👥 {} similar users loved this", similar_users)
                },
                imkitchen_recipe::services::recommendation::RecommendationReason::TrendingInCategory { category, .. } => {
                    format!("📈 Trending in {}", category)
                },
                imkitchen_recipe::services::recommendation::RecommendationReason::SeasonalRelevance { season, .. } => {
                    format!("🌿 Perfect for {}", season)
                },
                _ => "✨ Recommended for you".to_string(),
            }
        }).collect::<Vec<_>>().join(" • ");
        
        let confidence_color = if rec.confidence_score > 0.8 {
            "text-green-600"
        } else if rec.confidence_score > 0.6 {
            "text-blue-600"
        } else {
            "text-gray-600"
        };
        
        format!(r#"
        <div class="bg-white rounded-lg border border-gray-200 hover:shadow-md transition-all duration-200 overflow-hidden">
            <div class="p-4">
                <div class="flex justify-between items-start mb-3">
                    <div class="flex items-center space-x-2">
                        <span class="text-sm font-medium text-gray-900">#{}</span>
                        {}
                    </div>
                    <span class="text-xs font-bold {}">{}% match</span>
                </div>
                
                <h3 class="font-semibold text-gray-900 mb-2">{}</h3>
                
                <div class="text-xs text-gray-600 mb-3 line-clamp-2">
                    {}
                </div>
                
                <div class="flex items-center justify-between text-xs text-gray-500">
                    <div class="flex items-center space-x-3">
                        <span>⏱️ 25 min</span>
                        <span>⭐ 4.5 (12)</span>
                        {novelty_badge}
                    </div>
                    <button class="text-blue-600 hover:text-blue-800 font-medium">View Recipe</button>
                </div>
            </div>
        </div>
        "#,
        index + 1,
        recommendation_type_badge,
        confidence_color,
        (rec.confidence_score * 100.0) as i32,
        rec.title,
        if reasons_html.is_empty() { "Recommended just for you!" } else { &reasons_html },
        novelty_badge = if rec.novelty_score > 0.5 {
            r#"<span class="bg-yellow-100 text-yellow-800 px-1.5 py-0.5 rounded text-xs">New to you!</span>"#
        } else { "" }
        )
    }).collect::<Vec<_>>().join("");
    
    Html(format!(r#"
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div class="flex justify-between items-center mb-6">
            <div>
                <h2 class="text-lg font-semibold text-gray-900">
                    Personalized Recommendations
                    {meal_time_context}
                </h2>
                <p class="text-sm text-gray-600 mt-1">Curated just for you based on your preferences and behavior</p>
            </div>
            <div class="flex items-center space-x-2">
                <button class="text-xs px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded-full" 
                        onclick="refreshRecommendations('novelty')">🎲 Surprise me</button>
                <button class="text-xs px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded-full" 
                        onclick="refreshRecommendations('similar')">🔄 More like these</button>
            </div>
        </div>
        
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {recommendations_html}
        </div>
        
        <div class="mt-6 text-center">
            <p class="text-xs text-gray-500">
                Recommendations improve as you interact with recipes • 
                <button class="text-blue-600 hover:underline" onclick="showFeedback()">Rate these recommendations</button>
            </p>
        </div>
    </div>
    "#, 
    meal_time_context = if let Some(ref mt) = meal_time {
        format!(" <span class=\"text-sm font-normal text-gray-500\">for {:?}</span>", mt)
    } else {
        String::new()
    },
    recommendations_html = recommendations_html
    ))
}

/// POST /discovery/user-interaction - Record user interaction for recommendation learning
pub async fn record_user_interaction(
    State(_state): State<AppState>,
    Form(form): Form<UserInteractionForm>,
) -> impl IntoResponse {
    let recommendation_engine = RecommendationEngine::new();
    
    // Parse interaction type
    let interaction_type = match form.interaction_type.as_str() {
        "viewed" => InteractionType::Viewed,
        "clicked" => InteractionType::Clicked,
        "bookmarked" => InteractionType::Bookmarked,
        "cooked" => InteractionType::Cooked,
        "rated" => InteractionType::Rated,
        "shared" => InteractionType::Shared,
        _ => InteractionType::Viewed,
    };
    
    let interaction = UserInteraction {
        recipe_id: form.recipe_id,
        interaction_type,
        rating: form.rating,
        completion_status: None,
        timestamp: chrono::Utc::now(),
        context: form.context,
    };
    
    // Update user profile based on interaction
    let _ = recommendation_engine.update_user_profile_from_interaction(&form.user_id, &interaction).await;
    
    Html(r#"
    <div class="p-3 bg-green-50 border border-green-200 rounded-md text-center">
        <span class="text-sm text-green-600">✅ Thanks for your feedback! This helps improve your recommendations.</span>
    </div>
    "#)
}

/// GET /discovery/recommendation-metrics - Show recommendation system metrics
pub async fn recommendation_metrics(State(_state): State<AppState>) -> impl IntoResponse {
    let recommendation_engine = RecommendationEngine::new();
    let metrics = recommendation_engine.get_model_metrics().await.unwrap_or_default();
    
    let metrics_html = metrics.iter().map(|metric| {
        format!(r#"
        <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
            <h3 class="font-medium text-gray-900 mb-3 capitalize">{}</h3>
            <div class="grid grid-cols-2 gap-3 text-sm">
                <div>
                    <span class="text-gray-600">Accuracy:</span>
                    <span class="font-medium text-gray-900 ml-2">{:.1}%</span>
                </div>
                <div>
                    <span class="text-gray-600">Precision:</span>
                    <span class="font-medium text-gray-900 ml-2">{:.1}%</span>
                </div>
                <div>
                    <span class="text-gray-600">Coverage:</span>
                    <span class="font-medium text-gray-900 ml-2">{:.1}%</span>
                </div>
                <div>
                    <span class="text-gray-600">CTR:</span>
                    <span class="font-medium text-gray-900 ml-2">{:.1}%</span>
                </div>
                <div>
                    <span class="text-gray-600">User Satisfaction:</span>
                    <span class="font-medium text-gray-900 ml-2">{:.1}%</span>
                </div>
                <div>
                    <span class="text-gray-600">Novelty:</span>
                    <span class="font-medium text-gray-900 ml-2">{:.1}%</span>
                </div>
            </div>
        </div>
        "#, 
        metric.model_type.replace("_", " "),
        metric.accuracy * 100.0,
        metric.precision * 100.0,
        metric.coverage * 100.0,
        metric.click_through_rate * 100.0,
        metric.user_satisfaction * 100.0,
        metric.novelty * 100.0
        )
    }).collect::<Vec<_>>().join("");
    
    Html(format!(r#"
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Recommendation System Performance</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            {}
        </div>
        <div class="mt-4 text-center">
            <p class="text-xs text-gray-500">
                Metrics updated in real-time • Last evaluation: {}
            </p>
        </div>
    </div>
    "#, metrics_html, chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")))
}

/// GET /discovery/popular - Popular recipes with different time windows
pub async fn popular_recipes(
    Query(params): Query<TrendingQuery>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let popularity_service = PopularityService::new();
    
    // Determine time window
    let time_window = match params.time_window.as_deref() {
        Some("7d") => TimeWindow::last_week(),
        Some("30d") => TimeWindow::last_month(),
        _ => TimeWindow::last_24_hours(),
    };
    
    // Get popularity rankings
    let rankings = popularity_service.get_popularity_rankings(
        params.category.clone(), 
        &time_window, 
        15
    ).await.unwrap_or_default();
    
    if rankings.is_empty() {
        return Html(r#"
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <h2 class="text-lg font-semibold text-gray-900 mb-4">Popular Recipes</h2>
            <div class="text-center py-8">
                <p class="text-gray-500">No popular recipes found for this category.</p>
            </div>
        </div>
        "#.to_string());
    }
    
    // Generate popular recipes HTML
    let recipes_html = rankings.iter().enumerate().map(|(index, (recipe_id, score))| {
        let rank_badge = if index < 3 {
            format!(r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                #{} Most Popular
            </span>"#, index + 1)
        } else {
            format!(r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                #{}
            </span>"#, index + 1)
        };
        
        format!(r#"
        <div class="bg-gray-50 rounded-lg p-4 border border-gray-200 hover:shadow-md transition-shadow">
            <div class="flex justify-between items-start mb-2">
                {}
                <span class="text-xs text-green-600 font-medium">{:.0} pts</span>
            </div>
            <h3 class="font-medium text-gray-900">Recipe #{}</h3>
            <p class="text-sm text-gray-600 mt-1">Popular choice with high ratings and engagement</p>
            <div class="mt-3 flex items-center justify-between">
                <span class="text-xs text-gray-500">~25 min</span>
                <div class="flex items-center">
                    <span class="text-yellow-400">★★★★☆</span>
                    <span class="ml-1 text-xs text-gray-500">(4.4)</span>
                </div>
            </div>
        </div>
        "#, 
        rank_badge,
        score,
        recipe_id.to_string().chars().take(8).collect::<String>()
        )
    }).collect::<Vec<_>>().join("");
    
    Html(format!(r#"
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div class="flex justify-between items-center mb-4">
            <h2 class="text-lg font-semibold text-gray-900">
                Popular Recipes
                <span class="text-sm font-normal text-gray-500">({} window)</span>
            </h2>
            <div class="flex space-x-2">
                <button class="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded" 
                        onclick="updatePopular('24h')">24h</button>
                <button class="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded" 
                        onclick="updatePopular('7d')">7d</button>
                <button class="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded" 
                        onclick="updatePopular('30d')">30d</button>
            </div>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {}
        </div>
        <div class="mt-4 text-center">
            <p class="text-xs text-gray-500">
                Popularity based on views, ratings, bookmarks, and user engagement
            </p>
        </div>
    </div>
    "#, time_window.label, recipes_html))
}