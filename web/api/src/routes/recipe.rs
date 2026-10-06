use axum::{
    Json,
    extract::{FromRequest, Path, Query, State},
    http::StatusCode,
};
use imkitchen_web_shared::{
    AppState,
    services::recipe::{self, ShareError},
};

use crate::{
    ApiError, ApiResult,
    auth::{ApiChef, ApiUser},
    dto::{
        Created,
        recipe::{Browse, BrowseParams, Cook, CookParams, DetailDto, Exists, Page, page},
    },
};

impl From<ShareError> for ApiError {
    fn from(err: ShareError) -> Self {
        match err {
            ShareError::UsernameRequired => ApiError::UsernameRequired,
            ShareError::Core(err) => ApiError::from(err),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn browse(
    State(app): State<AppState>,
    user: ApiUser,
    Query(params): Query<BrowseParams>,
) -> ApiResult<Browse> {
    let browse = recipe::browse(&app, &user.id, params.into()).await?;
    Ok(Json(Browse {
        page: page(browse.recipes),
        has_shared: browse.has_shared,
    }))
}

/// Creates a draft (or resumes the unfinished one) and returns its id for
/// the editor.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn create(
    State(app): State<AppState>,
    user: ApiUser,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    let id = recipe::create_or_resume_draft(&app, &user.id, user.username.to_owned()).await?;
    Ok((StatusCode::CREATED, Json(Created { id })))
}

/// By id or slug. 404 unless owned or shared.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn detail(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id_or_slug,)): Path<(String,)>,
) -> ApiResult<DetailDto> {
    let id = recipe::resolve_id(&app, &id_or_slug).await?;
    let detail = recipe::detail(&app, &user.id, false, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(DetailDto::from(detail)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn similar(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id_or_slug,)): Path<(String,)>,
) -> ApiResult<Page> {
    let id = recipe::resolve_id(&app, &id_or_slug).await?;
    let view = app
        .core
        .recipe
        .user(&id)
        .await?
        .filter(|r| recipe::viewable(r, &user.id))
        .ok_or(ApiError::NotFound)?;
    Ok(Json(page(recipe::similar(&app, &view).await?)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn save(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    recipe::save(&app, &user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn unsave(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    recipe::unsave(&app, &user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn add_to_shopping(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    recipe::add_to_shopping(&app, &user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn share(
    State(app): State<AppState>,
    user: ApiChef,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    recipe::share(&app, &user.id, user.username.as_deref(), &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn unshare(
    State(app): State<AppState>,
    user: ApiChef,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    recipe::unshare(&app, &user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn share_all(State(app): State<AppState>, user: ApiChef) -> Result<StatusCode, ApiError> {
    recipe::share_all(&app, &user.id, user.username.as_deref()).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn unshare_all(
    State(app): State<AppState>,
    user: ApiChef,
) -> Result<StatusCode, ApiError> {
    recipe::unshare_all(&app, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletion is asynchronous: poll `GET /recipes/{id}/exists` until false.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    recipe::delete(&app, &user.id, &id).await?;
    Ok(StatusCode::ACCEPTED)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn exists(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> ApiResult<Exists> {
    let exists = recipe::exists(&app, &id).await?;
    Ok(Json(Exists { exists }))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn cook(
    State(app): State<AppState>,
    user: ApiUser,
    Path((username,)): Path<(String,)>,
    Query(params): Query<CookParams>,
) -> ApiResult<Cook> {
    let profile = recipe::cook_profile(&app, &username, params.into())
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(Cook::new(username, profile)))
}

use axum::extract::{Multipart, Request};
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::{
    ApiJson,
    dto::recipe::{Imported, RecipeInput, ThumbnailJson},
};

/// The editable fields; 403 for anyone but the owner.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn edit(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> ApiResult<RecipeInput> {
    let view = recipe::editable(&app, &user.id, &id)
        .await?
        .ok_or(ApiError::Forbidden)?;
    Ok(Json(RecipeInput::from(&view)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn update(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
    ApiJson(input): ApiJson<RecipeInput>,
) -> Result<StatusCode, ApiError> {
    recipe::update(&app, &user.id, input.into_update(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Batch import (same JSON shape as the web import). Recipes become visible
/// asynchronously: poll `GET /recipes/{last_id}/exists`.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn import(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(recipes): ApiJson<Vec<RecipeInput>>,
) -> Result<(StatusCode, Json<Imported>), ApiError> {
    let inputs = recipes.into_iter().map(RecipeInput::into_import).collect();
    let outcome = recipe::import_many(&app, &user.id, user.username.to_owned(), inputs).await;
    Ok((StatusCode::ACCEPTED, Json(Imported::from(outcome))))
}

/// Thumbnail upload, as `multipart/form-data` (first file field) or as JSON
/// `{content_type, data_base64}`. The resized variants appear asynchronously:
/// poll `GET /recipes/{id}` until `thumbnail_url` changes.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn thumbnail(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
    request: Request,
) -> Result<StatusCode, ApiError> {
    let content_type = request
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();

    let (image_type, data) = if content_type.starts_with("multipart/form-data") {
        let mut multipart = Multipart::from_request(request, &app)
            .await
            .map_err(|e| ApiError::User(e.body_text()))?;
        let field = multipart
            .next_field()
            .await
            .map_err(|e| ApiError::User(e.body_text()))?
            .ok_or_else(|| ApiError::User("No file provided".to_owned()))?;
        let image_type = field.content_type().unwrap_or("").to_owned();
        let data = field
            .bytes()
            .await
            .map_err(|e| ApiError::User(e.body_text()))?;
        (image_type, data.to_vec())
    } else {
        let ApiJson(json) = ApiJson::<ThumbnailJson>::from_request(request, &app).await?;
        let data = STANDARD
            .decode(json.data_base64.trim())
            .map_err(|_| ApiError::User("data_base64 is not valid base64".to_owned()))?;
        (json.content_type, data)
    };

    recipe::upload_thumbnail(&app, &user.id, &id, &image_type, data).await?;
    Ok(StatusCode::ACCEPTED)
}
