//! The read side of the recipes: the `recipes` table a user's list is read
//! from, and the subscription that keeps it current.
//!
//! The handler reloads the recipe from its stream and rewrites its row, so
//! an event delivered twice (the subscription is at-least-once) leaves the
//! row as it should be; a deleted recipe loses its row. Every node runs the
//! subscription over its own file, with a cursor of its own: a list is a
//! moment behind the command that changed it, on whichever node answers.

use evento::metadata::RawEvent;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::recipe::Recipe;
use timada_core::db::{FromRow, Json, search_key};
use timada_core::like_contains;
use timada_core::page::Cursor;

use super::Image;
use super::model::{self, RecipeState};
use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-api/recipes";

/// Follows the recipes into `db`. Start it over a handle that reads this
/// node's file as it stands: what the handler loads is behind the event it
/// handles.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_recipe_event())
}

#[evento::subscription_all]
async fn on_recipe_event<E: evento::Executor>(
    context: &Context<'_, E>,
    event: RawEvent<Recipe>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let Some(recipe) = model::load(context.executor, &event.aggregate_id).await? else {
        return Ok(());
    };
    if recipe.deleted {
        db.execute("DELETE FROM recipes WHERE id = ?1", (recipe.id.as_str(),))
            .await?;
        return Ok(());
    }
    upsert(&db, &recipe).await
}

/// Writes the row of a recipe as its stream leaves it.
async fn upsert(db: &Db, recipe: &RecipeState) -> anyhow::Result<()> {
    let content = &recipe.content;
    let image = recipe.image.as_ref().map(Image::from).map(Json);
    let image = image.as_ref().map(Json::to_text).transpose()?;
    db.execute(
        "INSERT INTO recipes (id, owner_id, title, description, prep_minutes, cook_minutes,
            servings, search, created_at, updated_at, image)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT (id) DO UPDATE SET
            title = excluded.title, description = excluded.description,
            prep_minutes = excluded.prep_minutes, cook_minutes = excluded.cook_minutes,
            servings = excluded.servings, search = excluded.search,
            updated_at = excluded.updated_at, image = excluded.image",
        (
            recipe.id.as_str(),
            recipe.owner_id.as_str(),
            content.title.as_str(),
            content.description.as_str(),
            content.prep_minutes,
            content.cook_minutes,
            content.servings,
            searched(recipe),
            recipe.created_at as i64,
            recipe.updated_at as i64,
            image,
        ),
    )
    .await?;
    Ok(())
}

/// What a search looks in: the title and the name of each ingredient, a
/// line each, without case or accents.
fn searched(recipe: &RecipeState) -> String {
    let content = &recipe.content;
    let lines: Vec<&str> = std::iter::once(content.title.as_str())
        .chain(content.ingredients.iter().map(|one| one.name.as_str()))
        .collect();
    search_key(&lines.join("\n"))
}

/// One row of `recipes`: what a list shows of a recipe.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RecipeRow {
    pub id: String,
    pub title: String,
    pub description: String,
    pub prep_minutes: i64,
    pub cook_minutes: i64,
    pub servings: i64,
    /// Unix seconds.
    pub created_at: i64,
    /// Unix seconds.
    pub updated_at: i64,
    /// The picture it shows, as the list answers it.
    pub image: Option<Json<Image>>,
}

impl RecipeRow {
    /// Where the row stands in its owner's list.
    pub fn cursor(&self) -> Cursor {
        Cursor {
            at: self.created_at,
            id: self.id.clone(),
        }
    }
}

/// The recipes of `owner_id`, the most recent first, `limit` at most: those
/// after `after` in the list, and only those `q` is found in (their title or
/// an ingredient, whatever the case and the accents) when there is one.
pub async fn recipes_of(
    db: &Db,
    owner_id: &str,
    q: Option<&str>,
    after: Option<&Cursor>,
    limit: i64,
) -> anyhow::Result<Vec<RecipeRow>> {
    let (at, id) = after.map_or((i64::MAX, ""), |cursor| (cursor.at, cursor.id.as_str()));
    db.fetch_all(
        "SELECT id, title, description, prep_minutes, cook_minutes, servings, created_at,
            updated_at, image
         FROM recipes INDEXED BY recipes_listed
         WHERE owner_id = ?1
            AND created_at <= ?2 AND (created_at < ?2 OR id < ?3)
            AND (?4 IS NULL OR search LIKE ?4 ESCAPE '\\')
         ORDER BY created_at DESC, id DESC LIMIT ?5",
        (
            owner_id,
            at,
            id,
            q.map(|q| like_contains(&search_key(q))),
            limit,
        ),
    )
    .await
}
