use imkitchen_core::recipe::query::user::UserView;
use imkitchen_types::recipe::{
    DietaryRestriction, Ingredient, IngredientCategory, IngredientUnit, IngredientUnitFormat,
    Instruction, RecipeType,
};
use imkitchen_types::shopping::RecipeStatus;
use serde::Serialize;

/// Relative URL of a recipe's thumbnail (the route is public), or `None`
/// when no image was uploaded. The client prefixes its API base URL.
pub fn thumbnail_url(id: &str, version: Option<&str>, device: &str) -> Option<String> {
    version.map(|v| format!("/recipes/{id}/thumbnail/{device}/image.webp?v={v}"))
}

/// `RecipeStatus` without serde's externally tagged `{"Cooking":3}` shape.
#[derive(Serialize, Clone, Debug)]
pub struct Status {
    /// `idle` | `cooking` | `completed`
    pub status: &'static str,
    /// Zero-based instruction index while `cooking`.
    pub step: Option<u8>,
}

impl From<&RecipeStatus> for Status {
    fn from(status: &RecipeStatus) -> Self {
        match status {
            RecipeStatus::Idle => Status {
                status: "idle",
                step: None,
            },
            RecipeStatus::Cooking(pos) => Status {
                status: "cooking",
                step: Some(*pos),
            },
            RecipeStatus::Completed => Status {
                status: "completed",
                step: None,
            },
        }
    }
}

/// An ingredient with its quantity already formatted for display (`250 g`,
/// `1.5 L`), the same way the templates do.
#[derive(Serialize, Clone, Debug)]
pub struct IngredientDto {
    pub name: String,
    pub quantity: u32,
    pub unit: Option<IngredientUnit>,
    pub category: Option<IngredientCategory>,
    pub quantity_label: String,
    /// Stable key, what `POST /groceries/toggle` expects.
    pub key: String,
}

impl From<&Ingredient> for IngredientDto {
    fn from(i: &Ingredient) -> Self {
        Self {
            name: i.name.to_owned(),
            quantity: i.quantity,
            unit: i.unit.clone(),
            category: i.category.clone(),
            quantity_label: i.unit.format(i.quantity),
            key: i.key(),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct InstructionDto {
    pub description: String,
    /// Minutes to wait before the next step (a timer), 0 when none.
    pub time_next: u16,
}

impl From<&Instruction> for InstructionDto {
    fn from(i: &Instruction) -> Self {
        Self {
            description: i.description.to_owned(),
            time_next: i.time_next,
        }
    }
}

/// A full recipe as the kitchen and the detail page show it.
#[derive(Serialize, Clone, Debug)]
pub struct Recipe {
    pub id: String,
    pub owner_id: String,
    pub owner_name: Option<String>,
    pub recipe_type: RecipeType,
    pub name: String,
    pub slug: String,
    pub origin: Option<String>,
    pub description: String,
    pub household_size: u16,
    pub prep_time: u16,
    pub cook_time: u16,
    pub total_time: u16,
    pub ingredients: Vec<IngredientDto>,
    pub instructions: Vec<InstructionDto>,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub accepts_accompaniment: bool,
    pub advance_prep: String,
    pub is_shared: bool,
    pub difficulty_score: u16,
    pub created_at: u64,
    pub thumbnail_url: Option<String>,
    pub blur_placeholder: Option<String>,
}

impl From<&UserView> for Recipe {
    fn from(r: &UserView) -> Self {
        Self {
            id: r.id.to_owned(),
            owner_id: r.owner_id.to_owned(),
            owner_name: r.owner_name.to_owned(),
            recipe_type: r.recipe_type.0.clone(),
            name: r.name.to_owned(),
            slug: r.slug.to_owned(),
            origin: r.origin.to_owned(),
            description: r.description.to_owned(),
            household_size: r.household_size,
            prep_time: r.prep_time,
            cook_time: r.cook_time,
            total_time: r.prep_time + r.cook_time,
            ingredients: r.ingredients.iter().map(IngredientDto::from).collect(),
            instructions: r.instructions.iter().map(InstructionDto::from).collect(),
            dietary_restrictions: r.dietary_restrictions.0.clone(),
            accepts_accompaniment: r.accepts_accompaniment,
            advance_prep: r.advance_prep.to_owned(),
            is_shared: r.is_shared,
            difficulty_score: r.difficulty_score,
            created_at: r.created_at,
            thumbnail_url: thumbnail_url(&r.id, r.thumbnail_version.as_deref(), "mobile"),
            blur_placeholder: r.blur_placeholder.to_owned(),
        }
    }
}

use evento::cursor::{ReadResult, Value};
use imkitchen_core::recipe::query::user::{SortBy, UserViewList};
use imkitchen_core::recipe::query::user_stat::UserStatView;
use imkitchen_web_shared::services::recipe::{BrowseQuery, CookProfile, CookQuery, Detail};
use serde::Deserialize;

/// A recipe card in a list.
#[derive(Serialize, Clone, Debug)]
pub struct Summary {
    pub id: String,
    pub owner_id: String,
    pub owner_name: Option<String>,
    pub recipe_type: RecipeType,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub total_time: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
    pub difficulty_score: u16,
    pub created_at: u64,
    pub thumbnail_url: Option<String>,
    pub blur_placeholder: Option<String>,
}

impl From<UserViewList> for Summary {
    fn from(r: UserViewList) -> Self {
        Self {
            thumbnail_url: thumbnail_url(&r.id, r.thumbnail_version.as_deref(), "mobile"),
            id: r.id,
            owner_id: r.owner_id,
            owner_name: r.owner_name,
            recipe_type: r.recipe_type.0,
            name: r.name,
            slug: r.slug,
            description: r.description,
            prep_time: r.prep_time,
            cook_time: r.cook_time,
            total_time: r.prep_time + r.cook_time,
            dietary_restrictions: r.dietary_restrictions.0,
            accepts_accompaniment: r.accepts_accompaniment,
            is_shared: r.is_shared,
            difficulty_score: r.difficulty_score,
            created_at: r.created_at,
            blur_placeholder: r.blur_placeholder,
        }
    }
}

/// Cursor page: `{"edges":[{"cursor","node"}],"page_info":{...}}`.
pub type Page = ReadResult<Summary>;

pub fn page(result: ReadResult<UserViewList>) -> Page {
    result.map(Summary::from)
}

/// `GET /recipes` query string.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct BrowseParams {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
    pub recipe_type: Option<RecipeType>,
    pub search: Option<String>,
    pub sort_by: Option<SortBy>,
    #[serde(default)]
    pub in_meal_plan: bool,
    #[serde(default)]
    pub mine: bool,
    #[serde(default)]
    pub no_image: bool,
}

impl From<BrowseParams> for BrowseQuery {
    fn from(p: BrowseParams) -> Self {
        Self {
            first: p.first,
            after: p.after,
            last: p.last,
            before: p.before,
            recipe_type: p.recipe_type,
            search: p.search.filter(|s| !s.trim().is_empty()),
            sort_by: p.sort_by.unwrap_or_default(),
            in_meal_plan: p.in_meal_plan,
            mine: p.mine,
            no_image: p.no_image,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Browse {
    pub page: Page,
    pub has_shared: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct Stat {
    pub total: u32,
    pub favorite: u32,
    pub shared: u32,
    pub from_community: u32,
}

impl From<&UserStatView> for Stat {
    fn from(s: &UserStatView) -> Self {
        Self {
            total: s.total,
            favorite: s.favorite,
            shared: s.shared,
            from_community: s.from_community,
        }
    }
}

/// The recipe page.
#[derive(Serialize, Debug)]
pub struct DetailDto {
    #[serde(flatten)]
    pub recipe: Recipe,
    pub is_owner: bool,
    pub saved: bool,
    pub in_shopping: bool,
    pub owner_description: String,
    pub owner_stat: Stat,
}

impl From<Detail> for DetailDto {
    fn from(d: Detail) -> Self {
        Self {
            recipe: Recipe::from(&d.recipe),
            is_owner: d.is_owner,
            saved: d.favorite.saved,
            in_shopping: d.in_shopping,
            owner_description: d.owner_description,
            owner_stat: Stat::from(&d.stat),
        }
    }
}

/// `GET /cooks/{username}` query string.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct CookParams {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
    pub recipe_type: Option<RecipeType>,
    pub search: Option<String>,
    pub sort_by: Option<SortBy>,
}

impl From<CookParams> for CookQuery {
    fn from(p: CookParams) -> Self {
        Self {
            first: p.first,
            after: p.after,
            last: p.last,
            before: p.before,
            recipe_type: p.recipe_type,
            search: p.search.filter(|s| !s.trim().is_empty()),
            sort_by: p.sort_by.unwrap_or_default(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Cook {
    pub username: String,
    pub description: String,
    pub stat: Stat,
    pub recipes: Page,
}

impl Cook {
    pub fn new(username: String, profile: CookProfile) -> Self {
        Self {
            username,
            description: profile.description,
            stat: Stat::from(&profile.stat),
            recipes: page(profile.recipes),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Exists {
    pub exists: bool,
}

use imkitchen_core::recipe::{ImportInput, UpdateInput};
use imkitchen_web_shared::services::recipe::ImportOutcome;

/// The editable fields of a recipe: what `GET /recipes/{id}/edit` returns and
/// `PUT /recipes/{id}` takes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecipeInput {
    pub recipe_type: RecipeType,
    pub name: String,
    #[serde(default)]
    pub origin: Option<String>,
    pub description: String,
    pub household_size: u16,
    pub prep_time: u16,
    pub cook_time: u16,
    #[serde(default)]
    pub ingredients: Vec<Ingredient>,
    #[serde(default)]
    pub instructions: Vec<Instruction>,
    #[serde(default)]
    pub dietary_restrictions: Vec<DietaryRestriction>,
    #[serde(default)]
    pub accepts_accompaniment: bool,
    #[serde(default)]
    pub advance_prep: String,
}

impl From<&UserView> for RecipeInput {
    fn from(r: &UserView) -> Self {
        Self {
            recipe_type: r.recipe_type.0.clone(),
            name: r.name.to_owned(),
            origin: r.origin.to_owned(),
            description: r.description.to_owned(),
            household_size: r.household_size,
            prep_time: r.prep_time,
            cook_time: r.cook_time,
            ingredients: r.ingredients.0.clone(),
            instructions: r.instructions.0.clone(),
            dietary_restrictions: r.dietary_restrictions.0.clone(),
            accepts_accompaniment: r.accepts_accompaniment,
            advance_prep: r.advance_prep.to_owned(),
        }
    }
}

impl RecipeInput {
    pub fn into_update(self, id: String) -> UpdateInput {
        UpdateInput {
            id,
            recipe_type: self.recipe_type,
            name: self.name,
            origin: self.origin.filter(|o| !o.trim().is_empty()),
            description: self.description,
            household_size: self.household_size,
            prep_time: self.prep_time,
            cook_time: self.cook_time,
            ingredients: self.ingredients,
            instructions: self.instructions,
            dietary_restrictions: self.dietary_restrictions,
            accepts_accompaniment: self.accepts_accompaniment,
            advance_prep: self.advance_prep,
        }
    }

    pub fn into_import(self) -> ImportInput {
        ImportInput {
            recipe_type: self.recipe_type,
            name: self.name,
            origin: self.origin.filter(|o| !o.trim().is_empty()),
            description: self.description,
            household_size: self.household_size,
            prep_time: self.prep_time,
            cook_time: self.cook_time,
            ingredients: self.ingredients,
            instructions: self.instructions,
            advance_prep: self.advance_prep,
            accepts_accompaniment: self.accepts_accompaniment,
            dietary_restrictions: self.dietary_restrictions,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ImportErrorDto {
    pub name: String,
    pub error: String,
}

/// `202` for a batch import: poll `GET /recipes/{last_id}/exists` until true.
#[derive(Serialize, Debug)]
pub struct Imported {
    pub last_id: Option<String>,
    pub errors: Vec<ImportErrorDto>,
}

impl From<ImportOutcome> for Imported {
    fn from(o: ImportOutcome) -> Self {
        Self {
            last_id: o.last_id,
            errors: o
                .errors
                .into_iter()
                .map(|e| ImportErrorDto {
                    name: e.name,
                    error: e.error,
                })
                .collect(),
        }
    }
}

/// JSON alternative to a multipart upload (the Lynx `fetch` has no FormData).
#[derive(Deserialize, Debug)]
pub struct ThumbnailJson {
    /// `image/png`, `image/jpeg` or `image/webp`.
    pub content_type: String,
    /// Standard base64 of the image bytes.
    pub data_base64: String,
}
