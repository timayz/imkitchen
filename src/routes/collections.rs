use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use recipe::{
    add_recipe_to_collection, create_collection, delete_collection, query_collections_by_user,
    remove_recipe_from_collection, update_collection, AddRecipeToCollectionCommand,
    CollectionReadModel, CreateCollectionCommand, DeleteCollectionCommand, RecipeError,
    RemoveRecipeFromCollectionCommand, UpdateCollectionCommand,
};

use crate::middleware::auth::Auth;
use crate::routes::auth::AppState;

pub struct CreateCollectionForm {
    pub name: String,
    pub description: Option<String>,
}

pub struct UpdateCollectionForm {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/collections.html")]
pub struct CollectionsTemplate {
    pub collections: Vec<CollectionReadModel>,
    pub error: String,
    pub user: Option<Auth>,
}

/// GET /collections - Display collections management page
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id))]
pub async fn get_collections(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
) -> impl IntoResponse {
    // Query user's collections from read model
    match query_collections_by_user(&auth.user_id, &state.db_pool).await {
        Ok(collections) => {
            tracing::info!(
                user_id = %auth.user_id,
                collection_count = collections.len(),
                "Fetched user collections"
            );
            let template = CollectionsTemplate {
                collections,
                error: String::new(),
                user: Some(auth),
            };
            Html(template.render().unwrap()).into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                error = ?e,
                "Failed to query collections"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load collections",
            )
                .into_response()
        }
    }
}

/// POST /collections - Handle collection creation form submission
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id))]
pub async fn post_create_collection(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    body: String,
) -> Response {
    // Parse URL-encoded form
    let form = match parse_create_collection_form(&body) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse create collection form");
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse form data: {}", e),
            )
                .into_response();
        }
    };

    tracing::info!(
        user_id = %auth.user_id,
        collection_name = %form.name,
        "Creating new collection"
    );

    // Create command
    let command = CreateCollectionCommand {
        name: form.name.clone(),
        description: form.description.clone(),
    };

    // Execute collection creation (evento event sourcing)
    match create_collection(command, &auth.user_id, &state.evento_executor).await {
        Ok(collection_id) => {
            tracing::info!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                collection_name = %form.name,
                "Collection created successfully"
            );
            // Redirect to collections page using TwinSpark (progressive enhancement)
            // Returns 200 OK with ts-location header for client-side navigation
            (StatusCode::OK, [("ts-location", "/collections")], ()).into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_name = %form.name,
                error = %msg,
                "Collection creation validation failed"
            );
            (StatusCode::UNPROCESSABLE_ENTITY, msg).into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                collection_name = %form.name,
                error = ?e,
                "Collection creation failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create collection. Please try again.",
            )
                .into_response()
        }
    }
}

/// POST /collections/{id}/update - Handle collection update form submission
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id, collection_id = %id))]
pub async fn post_update_collection(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(id): Path<String>,
    body: String,
) -> Response {
    // Parse URL-encoded form
    let form = match parse_update_collection_form(&body) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse update collection form");
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse form data: {}", e),
            )
                .into_response();
        }
    };

    tracing::info!(
        user_id = %auth.user_id,
        collection_id = %id,
        "Updating collection"
    );

    // Create command
    let command = UpdateCollectionCommand {
        collection_id: id.clone(),
        user_id: auth.user_id.clone(),
        name: form.name.clone(),
        description: form.description.clone().map(Some),
    };

    // Execute collection update (evento event sourcing)
    match update_collection(command, &state.evento_executor, &state.db_pool).await {
        Ok(()) => {
            tracing::info!(
                user_id = %auth.user_id,
                collection_id = %id,
                "Collection updated successfully"
            );
            // Redirect to collections page using TwinSpark
            (StatusCode::OK, [("ts-location", "/collections")], ()).into_response()
        }
        Err(RecipeError::PermissionDenied) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %id,
                "Collection update denied - permission error"
            );
            (StatusCode::FORBIDDEN, "You don't own this collection").into_response()
        }
        Err(RecipeError::NotFound) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %id,
                "Collection update failed - not found"
            );
            (StatusCode::NOT_FOUND, "Collection not found").into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %id,
                error = %msg,
                "Collection update validation failed"
            );
            (StatusCode::UNPROCESSABLE_ENTITY, msg).into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                collection_id = %id,
                error = ?e,
                "Collection update failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update collection. Please try again.",
            )
                .into_response()
        }
    }
}

/// POST /collections/{id}/delete - Handle collection deletion
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id, collection_id = %id))]
pub async fn post_delete_collection(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(id): Path<String>,
) -> Response {
    tracing::info!(
        user_id = %auth.user_id,
        collection_id = %id,
        "Deleting collection"
    );

    // Create command
    let command = DeleteCollectionCommand {
        collection_id: id.clone(),
        user_id: auth.user_id.clone(),
    };

    // Execute collection deletion (evento event sourcing - soft delete)
    match delete_collection(command, &state.evento_executor, &state.db_pool).await {
        Ok(()) => {
            tracing::info!(
                user_id = %auth.user_id,
                collection_id = %id,
                "Collection deleted successfully"
            );
            // Redirect to collections page using TwinSpark
            (StatusCode::OK, [("ts-location", "/collections")], ()).into_response()
        }
        Err(RecipeError::PermissionDenied) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %id,
                "Collection deletion denied - permission error"
            );
            (StatusCode::FORBIDDEN, "You don't own this collection").into_response()
        }
        Err(RecipeError::NotFound) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %id,
                "Collection deletion failed - not found"
            );
            (StatusCode::NOT_FOUND, "Collection not found").into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                collection_id = %id,
                error = ?e,
                "Collection deletion failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete collection. Please try again.",
            )
                .into_response()
        }
    }
}

/// POST /collections/{collection_id}/recipes/{recipe_id}/add - Add recipe to collection
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id, collection_id = %collection_id, recipe_id = %recipe_id))]
pub async fn post_add_recipe_to_collection(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path((collection_id, recipe_id)): Path<(String, String)>,
) -> Response {
    tracing::info!(
        user_id = %auth.user_id,
        collection_id = %collection_id,
        recipe_id = %recipe_id,
        "Adding recipe to collection"
    );

    // Create command
    let command = AddRecipeToCollectionCommand {
        collection_id: collection_id.clone(),
        recipe_id: recipe_id.clone(),
        user_id: auth.user_id.clone(),
    };

    // Execute recipe assignment (evento event sourcing)
    match add_recipe_to_collection(command, &state.evento_executor, &state.db_pool).await {
        Ok(()) => {
            tracing::info!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                recipe_id = %recipe_id,
                "Recipe added to collection successfully"
            );
            // Return success (idempotent - may already be in collection)
            (StatusCode::OK, "Recipe added to collection").into_response()
        }
        Err(RecipeError::PermissionDenied) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                recipe_id = %recipe_id,
                "Add recipe to collection denied - permission error"
            );
            (
                StatusCode::FORBIDDEN,
                "You don't own this collection or recipe",
            )
                .into_response()
        }
        Err(RecipeError::NotFound) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                recipe_id = %recipe_id,
                "Add recipe to collection failed - not found"
            );
            (StatusCode::NOT_FOUND, "Collection or recipe not found").into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                recipe_id = %recipe_id,
                error = ?e,
                "Add recipe to collection failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to add recipe to collection. Please try again.",
            )
                .into_response()
        }
    }
}

/// POST /collections/{collection_id}/recipes/{recipe_id}/remove - Remove recipe from collection
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id, collection_id = %collection_id, recipe_id = %recipe_id))]
pub async fn post_remove_recipe_from_collection(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path((collection_id, recipe_id)): Path<(String, String)>,
) -> Response {
    tracing::info!(
        user_id = %auth.user_id,
        collection_id = %collection_id,
        recipe_id = %recipe_id,
        "Removing recipe from collection"
    );

    // Create command
    let command = RemoveRecipeFromCollectionCommand {
        collection_id: collection_id.clone(),
        recipe_id: recipe_id.clone(),
        user_id: auth.user_id.clone(),
    };

    // Execute recipe removal (evento event sourcing)
    match remove_recipe_from_collection(command, &state.evento_executor, &state.db_pool).await {
        Ok(()) => {
            tracing::info!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                recipe_id = %recipe_id,
                "Recipe removed from collection successfully"
            );
            // Return success (idempotent - may already be removed)
            (StatusCode::OK, "Recipe removed from collection").into_response()
        }
        Err(RecipeError::PermissionDenied) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                recipe_id = %recipe_id,
                "Remove recipe from collection denied - permission error"
            );
            (StatusCode::FORBIDDEN, "You don't own this collection").into_response()
        }
        Err(RecipeError::NotFound) => {
            tracing::warn!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                recipe_id = %recipe_id,
                "Remove recipe from collection failed - not found"
            );
            (StatusCode::NOT_FOUND, "Collection not found").into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                collection_id = %collection_id,
                recipe_id = %recipe_id,
                error = ?e,
                "Remove recipe from collection failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to remove recipe from collection. Please try again.",
            )
                .into_response()
        }
    }
}

/// Parse create collection form from URL-encoded body
fn parse_create_collection_form(body: &str) -> Result<CreateCollectionForm, String> {
    use std::collections::HashMap;

    let mut fields: HashMap<String, Vec<String>> = HashMap::new();

    // Parse URL-encoded body
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            // Replace + with space before decoding (application/x-www-form-urlencoded uses + for space)
            let key_replaced = key.replace('+', " ");
            let value_replaced = value.replace('+', " ");
            let key = urlencoding::decode(&key_replaced).map_err(|e| e.to_string())?;
            let value = urlencoding::decode(&value_replaced).map_err(|e| e.to_string())?;

            fields
                .entry(key.to_string())
                .or_default()
                .push(value.to_string());
        }
    }

    // Extract name (required)
    let name = fields
        .get("name")
        .and_then(|v| v.first())
        .ok_or("Missing collection name")?
        .clone();

    // Extract description (optional)
    let description = fields
        .get("description")
        .and_then(|v| v.first())
        .cloned()
        .filter(|s| !s.is_empty());

    Ok(CreateCollectionForm { name, description })
}

/// Parse update collection form from URL-encoded body
fn parse_update_collection_form(body: &str) -> Result<UpdateCollectionForm, String> {
    use std::collections::HashMap;

    let mut fields: HashMap<String, Vec<String>> = HashMap::new();

    // Parse URL-encoded body
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            // Replace + with space before decoding (application/x-www-form-urlencoded uses + for space)
            let key_replaced = key.replace('+', " ");
            let value_replaced = value.replace('+', " ");
            let key = urlencoding::decode(&key_replaced).map_err(|e| e.to_string())?;
            let value = urlencoding::decode(&value_replaced).map_err(|e| e.to_string())?;

            fields
                .entry(key.to_string())
                .or_default()
                .push(value.to_string());
        }
    }

    // Extract name (optional)
    let name = fields
        .get("name")
        .and_then(|v| v.first())
        .cloned()
        .filter(|s| !s.is_empty());

    // Extract description (optional)
    let description = fields
        .get("description")
        .and_then(|v| v.first())
        .cloned()
        .filter(|s| !s.is_empty());

    Ok(UpdateCollectionForm { name, description })
}
