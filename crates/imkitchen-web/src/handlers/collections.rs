use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
    Form,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use imkitchen_recipe::domain::collection::CollectionPrivacy;
use imkitchen_recipe::domain::RecipeCategory;
use imkitchen_shared::Difficulty;

// Template struct for the main collections list page
#[derive(Template)]
#[template(path = "collections/CollectionList.html")]
struct CollectionListTemplate {
    collections: Vec<CollectionListItem>,
}

// Template struct for new collection page
#[derive(Template)]
#[template(path = "collections/NewCollection.html")]
struct NewCollectionTemplate {
    form_title: String,
    submit_button_text: String,
    collection: CollectionFormData,
    form_errors: CollectionFormErrors,
}

// Template struct for collection form (create/edit)
#[derive(Template)]
#[template(path = "collections/CollectionForm.html")]
struct CollectionFormTemplate {
    form_title: String,
    submit_button_text: String,
    collection: CollectionFormData,
    form_errors: CollectionFormErrors,
}

// Template struct for collections content fragment (for TwinSpark updates)
#[derive(Template)]
#[template(path = "fragments/collections/collections_content_fragment.html")]
struct CollectionsContentFragment {
    collections: Vec<CollectionListItem>,
}

// Template struct for collection detail page
#[derive(Template)]
#[template(path = "collections/CollectionDetail.html")]
struct CollectionDetailTemplate {
    collection: CollectionDetailData,
    is_owner: bool,
    average_difficulty: String,
    average_cook_time: u32,
}

// Template struct for favorites list page
#[derive(Template)]
#[template(path = "collections/FavoritesList.html")]
struct FavoritesListTemplate {
    favorites: UserFavoritesData,
}

// Template struct for recipe collection selector modal
#[derive(Template)]
#[template(path = "collections/RecipeCollectionSelector.html")]
struct RecipeCollectionSelectorTemplate {
    collection_id: Uuid,
    available_recipes: Vec<RecipeSelectorItem>,
    collection_recipe_count: usize,
}

// Data structures for templates
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used in templates and future implementations
struct CollectionListItem {
    collection_id: Uuid,
    name: String,
    description: String,
    privacy: CollectionPrivacy,
    recipe_count: usize,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_archived: bool,
}

#[derive(Debug, Clone, Default)]
struct CollectionFormData {
    name: String,
    description: String,
    privacy: String,
}

#[derive(Debug, Clone, Default)]
struct CollectionFormErrors {
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
struct CollectionDetailData {
    collection_id: Uuid,
    name: String,
    description: String,
    privacy: CollectionPrivacy,
    recipe_count: usize,
    recipes: Vec<RecipeDetailItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    categories: std::collections::HashMap<RecipeCategory, usize>,
}

#[derive(Debug, Clone)]
struct RecipeDetailItem {
    recipe_id: Uuid,
    title: String,
    difficulty: Difficulty,
    total_time_minutes: u32,
    rating: f64,
    review_count: u32,
    tags: Vec<String>,
    image_url: String,
}

#[derive(Debug, Clone)]
struct UserFavoritesData {
    total_count: usize,
    recipes: Vec<RecipeDetailItem>,
    quick_recipes: Vec<RecipeDetailItem>,
    highly_rated_favorites: Vec<RecipeDetailItem>,
    categories: std::collections::HashMap<RecipeCategory, usize>,
}

#[derive(Debug, Clone)]
struct RecipeSelectorItem {
    recipe_id: Uuid,
    title: String,
    difficulty: Difficulty,
    total_time_minutes: u32,
    rating: f64,
    image_url: String,
    in_collection: bool,
}

// Fragment templates for TwinSpark responses
#[derive(Template)]
#[template(path = "fragments/collections/collection_list_fragment.html")]
#[allow(dead_code)] // Used for TwinSpark fragment responses
struct CollectionListFragment {
    collections: Vec<CollectionListItem>,
}

#[derive(Template)]
#[template(path = "fragments/collections/collection_recipes_fragment.html")]
struct CollectionRecipesFragment {
    collection: CollectionDetailData,
    is_owner: bool,
}

// Form structures for TwinSpark requests
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCollectionForm {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Collection name must be between 1 and 100 characters"
    ))]
    name: String,
    #[validate(length(
        max = 500,
        message = "Collection description must be 500 characters or less"
    ))]
    description: Option<String>,
    privacy: String, // Will be validated against CollectionPrivacy enum
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields will be used when update functionality is implemented
pub struct UpdateCollectionForm {
    name: String,
    description: Option<String>,
    privacy: String,
}

#[derive(Debug, Deserialize)]
pub struct AddRecipeForm {
    recipe_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FilterParams {
    privacy: Option<String>,
    filter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SortParams {
    sort: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // context field used for advanced selector functionality
pub struct SelectorParams {
    collection_id: Uuid,
    context: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/collections/index.html")]
struct CollectionsTemplate {
    title: String,
}

/// GET /collections - Collections page
pub async fn collections_index(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = CollectionsTemplate {
        title: "My Collections".to_string(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /collections/list - Collections list page (once implemented)
pub async fn collections_list(
    State(_app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    // TODO: Fetch actual collections from the database
    let collections = vec![];

    // Check if this is a TwinSpark request (looking for text/html+partial accept header)
    let is_twinspark_request = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|accept| accept.contains("text/html+partial"))
        .unwrap_or(false);

    if is_twinspark_request {
        // Return content fragment for TwinSpark updates
        let template = CollectionsContentFragment { collections };
        match template.render() {
            Ok(html) => Ok(Html(html)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        // Return full page for direct navigation
        let template = CollectionListTemplate { collections };
        match template.render() {
            Ok(html) => Ok(Html(html)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

/// GET /collections/new - New collection form
pub async fn new_collection_form(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = NewCollectionTemplate {
        form_title: "Create New Collection".to_string(),
        submit_button_text: "Create Collection".to_string(),
        collection: CollectionFormData::default(),
        form_errors: CollectionFormErrors::default(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /collections/new/fragment - New collection form fragment for TwinSpark
pub async fn new_collection_form_fragment(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = CollectionFormTemplate {
        form_title: "Create New Collection".to_string(),
        submit_button_text: "Create Collection".to_string(),
        collection: CollectionFormData::default(),
        form_errors: CollectionFormErrors::default(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /favorites - User favorites page
pub async fn favorites_list(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Fetch actual favorites from the database
    let favorites = UserFavoritesData {
        total_count: 0,
        recipes: vec![],
        quick_recipes: vec![],
        highly_rated_favorites: vec![],
        categories: std::collections::HashMap::new(),
    };

    let template = FavoritesListTemplate { favorites };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// POST /collections - Create new collection
pub async fn create_collection(
    State(_app_state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CreateCollectionForm>,
) -> Result<Response<String>, StatusCode> {
    // Validate form data
    if let Err(_validation_errors) = form.validate() {
        // TODO: Return form with validation errors
        return Err(StatusCode::BAD_REQUEST);
    }

    // Parse privacy setting
    let _privacy = match form.privacy.as_str() {
        "Private" => CollectionPrivacy::Private,
        "Shared" => CollectionPrivacy::Shared,
        "Public" => CollectionPrivacy::Public,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // TODO: Create collection using command handler

    // Check if this is a TwinSpark request
    let is_twinspark_request = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|accept| accept.contains("text/html+partial"))
        .unwrap_or(false);

    if is_twinspark_request {
        // Use TwinSpark redirect via ts-location header
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("ts-location", "/collections")
            .body("".to_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(response)
    } else {
        // Use standard HTTP redirect for regular form submissions
        let response = Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header("location", "/collections")
            .body("".to_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(response)
    }
}

/// GET /collections/{id} - Show collection detail
pub async fn show_collection(
    State(_app_state): State<AppState>,
    Path(collection_id): Path<Uuid>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Fetch collection by ID from database/event store
    // For now, create mock data
    let collection = CollectionDetailData {
        collection_id,
        name: "Sample Collection".to_string(),
        description: "A sample collection for testing".to_string(),
        privacy: CollectionPrivacy::Private,
        recipe_count: 0,
        recipes: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        categories: std::collections::HashMap::new(),
    };

    let template = CollectionDetailTemplate {
        collection,
        is_owner: true, // TODO: Check actual ownership
        average_difficulty: "Easy".to_string(),
        average_cook_time: 0,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /collections/{id}/edit - Edit collection form
pub async fn edit_collection_form(
    State(_app_state): State<AppState>,
    Path(_collection_id): Path<Uuid>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Fetch collection by ID
    // For now, create mock form data
    let collection = CollectionFormData {
        name: "Sample Collection".to_string(),
        description: "A sample collection for testing".to_string(),
        privacy: "Private".to_string(),
    };

    let template = CollectionFormTemplate {
        form_title: "Edit Collection".to_string(),
        submit_button_text: "Update Collection".to_string(),
        collection,
        form_errors: CollectionFormErrors::default(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// PUT /collections/{id} - Update collection
pub async fn update_collection(
    State(_app_state): State<AppState>,
    Path(_collection_id): Path<Uuid>,
    Form(form): Form<UpdateCollectionForm>,
) -> Result<Html<String>, StatusCode> {
    // Parse privacy setting
    let _privacy = match form.privacy.as_str() {
        "Private" => CollectionPrivacy::Private,
        "Shared" => CollectionPrivacy::Shared,
        "Public" => CollectionPrivacy::Public,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // TODO: Update collection using command handler
    // For now, return updated collection list fragment
    let collections = vec![];

    let template = CollectionListTemplate { collections };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// DELETE /collections/{id} - Delete collection
pub async fn delete_collection(
    State(_app_state): State<AppState>,
    Path(_collection_id): Path<Uuid>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Delete collection using command handler
    // For now, return updated collection list fragment
    let collections = vec![];

    let template = CollectionListTemplate { collections };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// POST /collections/{id}/archive - Archive collection
pub async fn archive_collection(
    State(_app_state): State<AppState>,
    Path(_collection_id): Path<Uuid>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Archive collection using command handler
    // For now, return updated collection list fragment
    let collections = vec![];

    let template = CollectionListTemplate { collections };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// POST /collections/{id}/restore - Restore archived collection
pub async fn restore_collection(
    State(_app_state): State<AppState>,
    Path(_collection_id): Path<Uuid>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Restore collection using command handler
    // For now, return updated collection list fragment
    let collections = vec![];

    let template = CollectionListTemplate { collections };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /collections/search - Search collections
pub async fn search_collections(
    State(_app_state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Search collections using query handler
    let _query = params.q.unwrap_or_default();

    // For now, return empty collection list
    let collections = vec![];

    let template = CollectionListTemplate { collections };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /collections/filter - Filter collections
pub async fn filter_collections(
    State(_app_state): State<AppState>,
    Query(params): Query<FilterParams>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Filter collections using query handler
    let _privacy_filter = params.privacy;
    let _filter_type = params.filter;

    // For now, return empty collection list
    let collections = vec![];

    let template = CollectionListTemplate { collections };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /collections/sort - Sort collections
pub async fn sort_collections(
    State(_app_state): State<AppState>,
    Query(params): Query<SortParams>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Sort collections using query handler
    let _sort_order = params.sort;

    // For now, return empty collection list
    let collections = vec![];

    let template = CollectionListTemplate { collections };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// POST /collections/{id}/recipes - Add recipe to collection
pub async fn add_recipe_to_collection(
    State(_app_state): State<AppState>,
    Path(collection_id): Path<Uuid>,
    Form(form): Form<AddRecipeForm>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Add recipe to collection using command handler
    let _recipe_id = form.recipe_id;

    // Return updated recipe selector fragment
    let template = RecipeCollectionSelectorTemplate {
        collection_id,
        available_recipes: vec![],
        collection_recipe_count: 1, // Incremented
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// DELETE /collections/{id}/recipes/{recipe_id} - Remove recipe from collection
pub async fn remove_recipe_from_collection(
    State(_app_state): State<AppState>,
    Path((collection_id, _recipe_id)): Path<(Uuid, Uuid)>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Remove recipe from collection using command handler

    // Return updated collection recipes fragment
    let collection = CollectionDetailData {
        collection_id,
        name: "Sample Collection".to_string(),
        description: "Updated collection".to_string(),
        privacy: CollectionPrivacy::Private,
        recipe_count: 0, // Decremented
        recipes: vec![], // Remove the recipe
        created_at: Utc::now(),
        updated_at: Utc::now(),
        categories: std::collections::HashMap::new(),
    };

    let template = CollectionRecipesFragment {
        collection,
        is_owner: true,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// POST /favorites/{recipe_id} - Add recipe to favorites
pub async fn add_to_favorites(
    State(_app_state): State<AppState>,
    Path(_recipe_id): Path<Uuid>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Add recipe to favorites using command handler

    // Return updated favorites fragment
    let favorites = UserFavoritesData {
        total_count: 1, // Incremented
        recipes: vec![],
        quick_recipes: vec![],
        highly_rated_favorites: vec![],
        categories: std::collections::HashMap::new(),
    };

    let template = FavoritesListTemplate { favorites };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// DELETE /favorites/{recipe_id} - Remove recipe from favorites
pub async fn remove_from_favorites(
    State(_app_state): State<AppState>,
    Path(_recipe_id): Path<Uuid>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Remove recipe from favorites using command handler

    // Return updated favorites fragment
    let favorites = UserFavoritesData {
        total_count: 0, // Decremented
        recipes: vec![],
        quick_recipes: vec![],
        highly_rated_favorites: vec![],
        categories: std::collections::HashMap::new(),
    };

    let template = FavoritesListTemplate { favorites };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /recipes/selector - Recipe selector modal
pub async fn recipe_selector(
    State(_app_state): State<AppState>,
    Query(params): Query<SelectorParams>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Fetch available recipes from database
    // For now, create mock recipes
    let available_recipes = vec![
        RecipeSelectorItem {
            recipe_id: Uuid::new_v4(),
            title: "Quick Pasta Salad".to_string(),
            difficulty: Difficulty::Easy,
            total_time_minutes: 15,
            rating: 4.2,
            image_url: "".to_string(),
            in_collection: false,
        },
        RecipeSelectorItem {
            recipe_id: Uuid::new_v4(),
            title: "Grilled Chicken".to_string(),
            difficulty: Difficulty::Medium,
            total_time_minutes: 45,
            rating: 4.8,
            image_url: "".to_string(),
            in_collection: true,
        },
    ];

    let template = RecipeCollectionSelectorTemplate {
        collection_id: params.collection_id,
        available_recipes,
        collection_recipe_count: 1, // Count of recipes already in collection
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
