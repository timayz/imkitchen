//! Recipes: the library (browse), community detail, favorites, sharing and
//! the chef profile. Shared by the HTML routes and the JSON API.

use evento::cursor::{Args, ReadResult, Value};
use imkitchen_core::recipe::{
    favorite::Favorite,
    query::{
        user::{RecipesQuery, SortBy, UserView, UserViewList},
        user_stat::UserStatView,
    },
};
use imkitchen_types::recipe::RecipeType;

use crate::AppState;

/// Library filters, the same for the HTML page and the API.
#[derive(Debug, Default, Clone)]
pub struct BrowseQuery {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
    pub recipe_type: Option<RecipeType>,
    pub search: Option<String>,
    pub sort_by: SortBy,
    /// Only recipes already in the viewer's list ("Saved" on the web).
    pub in_meal_plan: bool,
    /// Only the viewer's own recipes.
    pub mine: bool,
    /// Only recipes without a thumbnail.
    pub no_image: bool,
}

pub struct Browse {
    pub recipes: ReadResult<UserViewList>,
    /// Drives the Share All / Make All Private toggle on the chef toolbar.
    /// Only considers the loaded page of recipes.
    pub has_shared: bool,
}

/// The library: the community (shared recipes not yet in the viewer's list,
/// filtered by their dietary restrictions) or, with `mine`, the viewer's own.
pub async fn browse(app: &AppState, user_id: &str, q: BrowseQuery) -> anyhow::Result<Browse> {
    let args = Args {
        first: q.first,
        after: q.after,
        last: q.last,
        before: q.before,
    };

    let dietary_restrictions = if q.in_meal_plan || q.mine {
        vec![]
    } else {
        app.identity
            .meal_preferences
            .load(user_id)
            .await?
            .dietary_restrictions
    };

    let (owner, is_shared) = if q.mine {
        (Some(user_id.to_owned()), None)
    } else {
        (None, Some(true))
    };

    // `in_meal_plan: Some((_, false))` is an exclusion (NOT EXISTS), used by the
    // community browse default to hide already-planned recipes from the picker.
    // When filtering by ownership, that exclusion would hide every recipe the
    // user has already added — drop the plan filter unless In plan is on.
    let in_meal_plan = if q.in_meal_plan {
        Some((user_id.to_owned(), true))
    } else if q.mine {
        None
    } else {
        Some((user_id.to_owned(), false))
    };

    let has_thumbnail = if q.no_image { Some(false) } else { None };

    let recipes = app
        .core
        .recipe
        .filter_user(RecipesQuery {
            exclude_ids: None,
            user_id: owner,
            recipe_type: q.recipe_type,
            is_shared,
            has_thumbnail,
            dietary_restrictions,
            dietary_where_any: false,
            in_meal_plan,
            sort_by: q.sort_by,
            args: args.limit(20),
            search: q.search,
        })
        .await?;

    let has_shared = recipes.edges.iter().any(|r| r.node.is_shared);

    Ok(Browse {
        recipes,
        has_shared,
    })
}

/// Reuses the user's unfinished draft or creates a new one.
pub async fn create_or_resume_draft(
    app: &AppState,
    user_id: &str,
    username: Option<String>,
) -> imkitchen_core::Result<String> {
    match app.core.recipe.find_user_draft(user_id).await? {
        Some(id) => Ok(id),
        None => app.core.recipe.create(user_id, username).await,
    }
}

/// Resolves a path segment as a slug, falling back to a raw recipe id so
/// legacy/id-shaped links keep working.
pub async fn resolve_id(app: &AppState, slug_or_id: &str) -> anyhow::Result<String> {
    Ok(app
        .core
        .recipe
        .find_id_by_slug(slug_or_id)
        .await?
        .unwrap_or_else(|| slug_or_id.to_owned()))
}

/// Whether `viewer_id` may see the recipe: owners always, others only when
/// it is shared with the community.
pub fn viewable(recipe: &UserView, viewer_id: &str) -> bool {
    recipe.owner_id == viewer_id || recipe.is_shared
}

pub struct Detail {
    pub recipe: UserView,
    pub stat: UserStatView,
    pub favorite: Favorite,
    pub owner_description: String,
    /// Whether this recipe is already in the viewer's shopping list.
    pub in_shopping: bool,
    pub is_owner: bool,
}

/// The recipe page. `None` when it does not exist or the viewer may not see
/// it. `anonymous` viewers (the demo identity) have no shopping list.
pub async fn detail(
    app: &AppState,
    viewer_id: &str,
    anonymous: bool,
    id: &str,
) -> anyhow::Result<Option<Detail>> {
    let Some(recipe) = app.core.recipe.user(id).await? else {
        return Ok(None);
    };
    if !viewable(&recipe, viewer_id) {
        return Ok(None);
    }

    let stat = app
        .core
        .recipe
        .find_user_stat(&recipe.owner_id)
        .await?
        .unwrap_or_default();
    let favorite = app.core.recipe.favorite.load(&recipe.id, viewer_id).await?;
    let owner_profile = app.identity.user_profile.load(&recipe.owner_id).await?;

    let in_shopping = if anonymous {
        false
    } else {
        app.core
            .shopping
            .load(viewer_id)
            .await?
            .map(|s| s.recipes.contains(&recipe.id))
            .unwrap_or(false)
    };

    Ok(Some(Detail {
        is_owner: recipe.owner_id == viewer_id,
        recipe,
        stat,
        favorite,
        owner_description: owner_profile.description,
        in_shopping,
    }))
}

/// Up to 10 shared recipes of the same course, in three fallback tiers: same
/// dietary restrictions (all), then any of them, then none.
pub async fn similar(
    app: &AppState,
    recipe: &UserView,
) -> anyhow::Result<ReadResult<UserViewList>> {
    let exclude_ids = vec![recipe.id.to_owned()];
    let query = |exclude: Vec<String>, restrictions: Vec<_>, any: bool| RecipesQuery {
        exclude_ids: Some(exclude),
        user_id: None,
        recipe_type: Some(recipe.recipe_type.0.to_owned()),
        is_shared: Some(true),
        has_thumbnail: None,
        dietary_restrictions: restrictions,
        dietary_where_any: any,
        in_meal_plan: None,
        sort_by: SortBy::Random,
        args: Args::forward(10, None),
        search: None,
    };

    let mut similar_recipes = app
        .core
        .recipe
        .filter_user(query(
            exclude_ids.clone(),
            recipe.dietary_restrictions.0.to_vec(),
            false,
        ))
        .await?;

    if similar_recipes.edges.len() < 10 {
        let mut similar_ids: Vec<String> = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect();
        similar_ids.extend(exclude_ids.clone());
        let more = app
            .core
            .recipe
            .filter_user(query(
                similar_ids,
                recipe.dietary_restrictions.0.to_vec(),
                true,
            ))
            .await?;
        similar_recipes.edges.extend(more.edges);
    }

    if similar_recipes.edges.len() < 10 {
        let mut similar_ids: Vec<String> = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect();
        similar_ids.extend(exclude_ids);
        let more = app
            .core
            .recipe
            .filter_user(query(similar_ids, vec![], false))
            .await?;
        similar_recipes.edges.extend(more.edges);
    }

    // Fallback tiers can overshoot the target; keep at most 10 (exact matches,
    // which are collected first, take precedence).
    similar_recipes.edges.truncate(10);

    Ok(similar_recipes)
}

/// Favorite a community recipe. Not-found when it is not shared.
pub async fn save(app: &AppState, user_id: &str, id: &str) -> imkitchen_core::Result<()> {
    let Some(recipe) = app.core.recipe.user(id).await? else {
        imkitchen_core::not_found!("recipe");
    };
    if !recipe.is_shared {
        imkitchen_core::not_found!("recipe");
    }
    app.core
        .recipe
        .favorite
        .save(id, recipe.owner_id, user_id)
        .await
}

pub async fn unsave(app: &AppState, user_id: &str, id: &str) -> imkitchen_core::Result<()> {
    app.core.recipe.favorite.unsave(id, user_id).await
}

/// Add the recipe to the viewer's shopping list. Any viewable recipe (owned
/// or shared) can be added; scaling uses the viewer's household size.
pub async fn add_to_shopping(
    app: &AppState,
    user_id: &str,
    id: &str,
) -> imkitchen_core::Result<()> {
    let Some(recipe) = app.core.recipe.user(id).await? else {
        imkitchen_core::not_found!("recipe");
    };
    if !viewable(&recipe, user_id) {
        imkitchen_core::not_found!("recipe");
    }
    let preferences = app.identity.meal_preferences.load(user_id).await?;
    app.core
        .shopping
        .add_recipe(id, preferences.household_size, user_id)
        .await
}

/// Sharing needs a public name for the chef.
#[derive(Debug)]
pub enum ShareError {
    UsernameRequired,
    Core(imkitchen_core::Error),
}

impl From<imkitchen_core::Error> for ShareError {
    fn from(err: imkitchen_core::Error) -> Self {
        ShareError::Core(err)
    }
}

pub async fn share(
    app: &AppState,
    user_id: &str,
    username: Option<&str>,
    id: &str,
) -> Result<(), ShareError> {
    let username = username.ok_or(ShareError::UsernameRequired)?;
    app.core
        .recipe
        .share_to_community(id, user_id, username)
        .await?;
    Ok(())
}

pub async fn unshare(app: &AppState, user_id: &str, id: &str) -> imkitchen_core::Result<()> {
    app.core.recipe.make_private(id, user_id).await
}

pub async fn share_all(
    app: &AppState,
    user_id: &str,
    username: Option<&str>,
) -> Result<(), ShareError> {
    let username = username.ok_or(ShareError::UsernameRequired)?;
    app.core
        .recipe
        .share_all_to_community(user_id, username)
        .await?;
    Ok(())
}

pub async fn unshare_all(app: &AppState, user_id: &str) -> imkitchen_core::Result<()> {
    app.core.recipe.make_all_private(user_id).await
}

pub async fn delete(app: &AppState, user_id: &str, id: &str) -> imkitchen_core::Result<()> {
    app.core.recipe.delete(id, user_id).await
}

/// Whether the recipe is (still) in the user projection: deletion is done
/// when this turns false, an import is ready when it turns true.
pub async fn exists(app: &AppState, id: &str) -> anyhow::Result<bool> {
    Ok(app.core.recipe.find_user(id).await?.is_some())
}

/// Public profile filters.
#[derive(Debug, Default, Clone)]
pub struct CookQuery {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
    pub recipe_type: Option<RecipeType>,
    pub search: Option<String>,
    pub sort_by: SortBy,
}

pub struct CookProfile {
    pub owner_id: String,
    pub stat: UserStatView,
    pub description: String,
    pub recipes: ReadResult<UserViewList>,
}

/// A chef's shared recipes. `None` when no chef carries the username.
pub async fn cook_profile(
    app: &AppState,
    username: &str,
    q: CookQuery,
) -> anyhow::Result<Option<CookProfile>> {
    let Some(owner_id) = app.core.recipe.find_owner_id_by_name(username).await? else {
        return Ok(None);
    };

    let stat = app
        .core
        .recipe
        .find_user_stat(&owner_id)
        .await?
        .unwrap_or_default();
    let owner_profile = app.identity.user_profile.load(&owner_id).await?;

    let args = Args {
        first: q.first,
        after: q.after,
        last: q.last,
        before: q.before,
    };
    let recipes = app
        .core
        .recipe
        .filter_user(RecipesQuery {
            exclude_ids: None,
            user_id: Some(owner_id.to_owned()),
            recipe_type: q.recipe_type,
            is_shared: Some(true),
            has_thumbnail: None,
            dietary_restrictions: vec![],
            dietary_where_any: false,
            in_meal_plan: None,
            sort_by: q.sort_by,
            args: args.limit(20),
            search: q.search,
        })
        .await?;

    Ok(Some(CookProfile {
        owner_id,
        stat,
        description: owner_profile.description,
        recipes,
    }))
}

use imkitchen_core::recipe::{ImportInput, UpdateInput};

/// The recipe for its editor: owners only (`None` when missing or not owned).
pub async fn editable(app: &AppState, user_id: &str, id: &str) -> anyhow::Result<Option<UserView>> {
    Ok(app
        .core
        .recipe
        .user(id)
        .await?
        .filter(|r| r.owner_id == user_id))
}

pub async fn update(
    app: &AppState,
    user_id: &str,
    input: UpdateInput,
) -> imkitchen_core::Result<()> {
    app.core.recipe.update(input, user_id).await
}

/// One failed recipe of a batch import.
#[derive(Debug, Clone)]
pub struct ImportError {
    pub name: String,
    pub error: String,
}

/// What a batch import produced: the id of the last imported recipe (to poll
/// for readiness) and the recipes that were rejected.
#[derive(Debug, Default)]
pub struct ImportOutcome {
    pub last_id: Option<String>,
    pub errors: Vec<ImportError>,
}

/// Imports recipes one by one; a rejected recipe does not stop the batch.
pub async fn import_many(
    app: &AppState,
    user_id: &str,
    username: Option<String>,
    recipes: Vec<ImportInput>,
) -> ImportOutcome {
    let mut outcome = ImportOutcome::default();
    for recipe in recipes {
        let name = recipe.name.to_owned();
        match app
            .core
            .recipe
            .import(recipe, user_id, username.to_owned())
            .await
        {
            Ok(id) => outcome.last_id = Some(id),
            Err(imkitchen_core::Error::Server(err)) => {
                tracing::error!(user = user_id, err = %err, "failed to import recipes");
                outcome.errors.push(ImportError {
                    name,
                    error: crate::template::SERVER_ERROR_MESSAGE.to_owned(),
                });
            }
            Err(error) => outcome.errors.push(ImportError {
                name,
                error: error.to_string(),
            }),
        }
    }
    outcome
}

pub const THUMBNAIL_TYPES: [&str; 3] = ["image/png", "image/jpeg", "image/webp"];

/// Stores a new thumbnail (resized asynchronously). Rejects unsupported
/// content types before touching the domain.
pub async fn upload_thumbnail(
    app: &AppState,
    user_id: &str,
    id: &str,
    content_type: &str,
    data: Vec<u8>,
) -> imkitchen_core::Result<()> {
    if !THUMBNAIL_TYPES.contains(&content_type) {
        imkitchen_core::user!("Invalid file type: {content_type}");
    }
    app.core.recipe.upload_thumbnail(id, data, user_id).await
}
