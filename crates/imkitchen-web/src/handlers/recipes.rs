use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use imkitchen_recipe::{
    domain::{Difficulty, Ingredient, Recipe, RecipeCategory},
    projections::{RecipeDetailView, RecipeListView},
    queries::RecipeSearchQuery,
};

// Simple template for basic functionality
#[derive(Template)]
#[template(
    source = "<h1>{{ title }}</h1><p>Recipe management coming soon!</p>",
    ext = "html"
)]
pub struct SimpleTemplate {
    pub title: String,
}

// Template for recipe discovery page with full accessibility
#[derive(Template)]
#[template(path = "pages/recipes/discover.html")]
pub struct RecipeDiscoverTemplate {
    pub title: String,
}

// Form data structures
#[derive(Deserialize)]
pub struct CreateRecipeForm {
    pub title: String,
    pub category: String,
    pub difficulty: String,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub tags: Option<String>,
    pub is_public: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub search_query: Option<String>,
    pub category: Option<String>,
    pub difficulty: Option<String>,
    pub max_time: Option<String>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

// Recipe CRUD Handlers

/// GET /recipes - List all recipes
pub async fn list_recipes(
    Query(params): Query<SearchQuery>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Build search query from parameters
    let mut query = RecipeSearchQuery::default();

    if let Some(search_text) = params.search_query.as_ref() {
        if !search_text.trim().is_empty() {
            query = query.with_search_text(search_text.clone());
        }
    }

    if let Some(category_str) = params.category.as_ref() {
        if let Ok(category) =
            serde_json::from_str::<RecipeCategory>(&format!("\"{}\"", category_str))
        {
            query = query.with_category(category);
        }
    }

    if let Some(difficulty_str) = params.difficulty.as_ref() {
        if let Ok(difficulty) =
            serde_json::from_str::<Difficulty>(&format!("\"{}\"", difficulty_str))
        {
            query = query.with_difficulty(difficulty);
        }
    }

    if let Some(max_time_str) = params.max_time.as_ref() {
        if let Ok(max_time) = max_time_str.parse::<u32>() {
            query = query
                .with_max_prep_time(max_time)
                .with_max_cook_time(max_time);
        }
    }

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    let offset = (page - 1) * limit;
    let _query = query.with_pagination(limit, offset);

    // Mock data for now - in real implementation, this would query the database
    let _recipes = create_mock_recipe_list();

    let template = RecipeDiscoverTemplate {
        title: "Discover Recipes".to_string(),
    };

    Html(template.render().unwrap())
}

/// GET /recipes/new - Show create recipe form
pub async fn new_recipe_form(State(_state): State<AppState>) -> impl IntoResponse {
    let template = SimpleTemplate {
        title: "Create New Recipe".to_string(),
    };

    Html(template.render().unwrap())
}

/// POST /recipes - Create a new recipe
pub async fn create_recipe(
    State(_state): State<AppState>,
    Form(form): Form<CreateRecipeForm>,
) -> impl IntoResponse {
    // Validate and create recipe
    let mut form_errors = std::collections::HashMap::new();

    // Basic validation
    if form.title.trim().is_empty() || form.title.len() > 200 {
        form_errors.insert(
            "title".to_string(),
            "Title must be 1-200 characters".to_string(),
        );
    }

    if !form_errors.is_empty() {
        // Return form with errors
        let template = SimpleTemplate {
            title: "Recipe Form Errors".to_string(),
        };

        return Html(template.render().unwrap());
    }

    // Create command - in real implementation, this would use the command handler
    let recipe_id = Uuid::new_v4();

    // For now, redirect to recipe detail (in real implementation, this would be handled by command processing)
    Html(format!(
        r#"<div ts-req="/recipes/{}" ts-target="body">Recipe created successfully! Redirecting...</div>"#,
        recipe_id
    ))
}

/// GET /recipes/:id - Show recipe detail
pub async fn show_recipe(
    Path(recipe_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // In real implementation, query the recipe by ID
    let recipe = create_mock_recipe_detail(recipe_id);

    let template = SimpleTemplate {
        title: format!("Recipe Details: {}", recipe.title),
    };

    Html(template.render().unwrap())
}

/// GET /recipes/:id/edit - Show edit recipe form
pub async fn edit_recipe_form(
    Path(recipe_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // In real implementation, query the recipe by ID
    let recipe = create_mock_recipe(recipe_id);

    let template = SimpleTemplate {
        title: format!("Edit Recipe: {}", recipe.title),
    };

    Html(template.render().unwrap())
}

/// PUT /recipes/:id - Update recipe
pub async fn update_recipe(
    Path(_recipe_id): Path<Uuid>,
    State(_state): State<AppState>,
    Form(_form): Form<CreateRecipeForm>,
) -> impl IntoResponse {
    // Similar validation and processing as create_recipe
    // In real implementation, this would use UpdateRecipeCommand

    Html(
        r#"<div ts-req="/recipes" ts-target="body">Recipe updated successfully! Redirecting...</div>"#,
    )
}

/// DELETE /recipes/:id - Delete recipe
pub async fn delete_recipe(
    Path(_recipe_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // In real implementation, this would use DeleteRecipeCommand

    Html(
        r#"<div ts-req="/recipes" ts-target="body">Recipe deleted successfully! Redirecting...</div>"#,
    )
}

// TwinSpark Fragment Handlers

/// POST /recipes/ingredients/add - Add ingredient via TwinSpark
pub async fn add_ingredient(State(_state): State<AppState>) -> impl IntoResponse {
    // Return a new ingredient form row
    let _new_ingredient = Ingredient::new(String::new(), 1.0, String::new(), None).unwrap();

    // In real implementation, this would return just the ingredient row HTML fragment
    Html(
        r#"
    <div class="flex items-center space-x-3 p-4 bg-gray-50 rounded-lg border border-gray-200" data-ingredient-index="new">
        <div class="flex-1">
            <input type="text" name="ingredients[new][name]" placeholder="Ingredient name" 
                   class="w-full px-3 py-2 border border-gray-300 rounded focus:border-blue-500 focus:ring-1 focus:ring-blue-200" required />
        </div>
        <div class="w-24">
            <input type="number" name="ingredients[new][quantity]" placeholder="2" step="0.01" min="0.01"
                   class="w-full px-3 py-2 border border-gray-300 rounded focus:border-blue-500 focus:ring-1 focus:ring-blue-200" required />
        </div>
        <div class="w-20">
            <input type="text" name="ingredients[new][unit]" placeholder="cups"
                   class="w-full px-3 py-2 border border-gray-300 rounded focus:border-blue-500 focus:ring-1 focus:ring-blue-200" required />
        </div>
        <button type="button" class="p-2 text-red-500 hover:text-red-700 hover:bg-red-50 rounded transition-colors duration-200">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
            </svg>
        </button>
    </div>
    "#,
    )
}

/// POST /recipes/ingredients/remove - Remove ingredient via TwinSpark
pub async fn remove_ingredient(State(_state): State<AppState>) -> impl IntoResponse {
    // Return updated ingredients list without the specified ingredient
    // In real implementation, this would filter out the ingredient and re-render the list

    Html(r#"<div class="text-center py-4 text-green-600">Ingredient removed successfully!</div>"#)
}

/// POST /recipes/instructions/add - Add instruction via TwinSpark
pub async fn add_instruction(State(_state): State<AppState>) -> impl IntoResponse {
    // Return a new instruction form row
    Html(
        r#"
    <div class="flex items-start space-x-3 p-4 bg-gray-50 rounded-lg border border-gray-200" data-instruction-index="new">
        <div class="flex-shrink-0 w-8 h-8 bg-purple-100 text-purple-600 rounded-full flex items-center justify-center font-semibold text-sm">
            #
        </div>
        <div class="flex-1">
            <textarea name="instructions[new][text]" placeholder="Describe this cooking step..." rows="3"
                      class="w-full px-3 py-2 border border-gray-300 rounded focus:border-blue-500 focus:ring-1 focus:ring-blue-200 resize-none" required></textarea>
        </div>
        <div class="w-20">
            <input type="number" name="instructions[new][estimated_minutes]" placeholder="5" min="1"
                   class="w-full px-3 py-2 border border-gray-300 rounded focus:border-blue-500 focus:ring-1 focus:ring-blue-200" />
            <label class="text-xs text-gray-500 mt-1 block">min</label>
        </div>
        <button type="button" class="p-2 text-red-500 hover:text-red-700 hover:bg-red-50 rounded transition-colors duration-200">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
            </svg>
        </button>
    </div>
    "#,
    )
}

/// POST /recipes/instructions/remove - Remove instruction via TwinSpark
pub async fn remove_instruction(State(_state): State<AppState>) -> impl IntoResponse {
    // Return updated instructions list without the specified instruction
    Html(r#"<div class="text-center py-4 text-green-600">Instruction removed successfully!</div>"#)
}

/// GET /recipes/search - Live search via TwinSpark
pub async fn search_recipes(
    Query(params): Query<SearchQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Delegate to list_recipes but return only the grid portion
    list_recipes(Query(params), State(state)).await
}

/// GET/POST /recipes/filter - Filter recipes via TwinSpark
pub async fn filter_recipes(
    Query(params): Query<SearchQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Delegate to list_recipes
    list_recipes(Query(params), State(state)).await
}

// Helper functions

fn create_mock_recipe(_recipe_id: Uuid) -> Recipe {
    Recipe::new(
        "Sample Recipe".to_string(),
        vec![],
        vec![],
        15,
        30,
        Difficulty::Medium,
        RecipeCategory::Dessert,
        Uuid::new_v4(),
        true,
        vec!["sample".to_string()],
    )
    .unwrap()
}

fn create_mock_recipe_list() -> Vec<RecipeListView> {
    vec![]
}

fn create_mock_recipe_detail(recipe_id: Uuid) -> RecipeDetailView {
    use chrono::Utc;

    RecipeDetailView {
        recipe_id,
        title: "Sample Recipe".to_string(),
        ingredients: vec![],
        instructions: vec![],
        prep_time_minutes: 10,
        cook_time_minutes: 20,
        difficulty: Difficulty::Medium,
        category: RecipeCategory::Main,
        rating: 4.6,
        review_count: 89,
        created_by: Uuid::new_v4(),
        is_public: true,
        tags: vec!["sample".to_string()],
        created_at: Utc::now(),
        updated_at: None,
        image_url: None,
        nutritional_info: None,
    }
}
