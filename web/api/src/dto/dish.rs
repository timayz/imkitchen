use imkitchen_core::dish::query::Candidate;
use imkitchen_types::recipe::RecipeType;
use imkitchen_web_shared::services::dish::{ComposeInput, DishRecipe, DishView};
use serde::{Deserialize, Serialize};

pub use super::Created;
use super::recipe::thumbnail_url;

#[derive(Serialize, Clone, Debug)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub recipe_type: RecipeType,
    pub prep_time: u16,
    pub cook_time: u16,
    pub total_time: u16,
    pub thumbnail_url: Option<String>,
    pub blur_placeholder: Option<String>,
}

impl From<&DishRecipe> for Recipe {
    fn from(r: &DishRecipe) -> Self {
        Self {
            id: r.id.to_owned(),
            name: r.name.to_owned(),
            slug: r.slug.to_owned(),
            recipe_type: r.recipe_type.clone(),
            prep_time: r.prep_time,
            cook_time: r.cook_time,
            total_time: r.total_time(),
            thumbnail_url: thumbnail_url(&r.id, r.thumbnail_version.as_deref(), "mobile"),
            blur_placeholder: r.blur_placeholder.to_owned(),
        }
    }
}

/// A composed dish: its recipes in meal order (starter, main, sides,
/// dessert, drink, sauces), deleted recipes left out.
#[derive(Serialize, Clone, Debug)]
pub struct Dish {
    pub id: String,
    pub name: String,
    pub recipes: Vec<Recipe>,
    /// Every recipe of the dish is already in the kitchen list.
    pub in_list: bool,
    /// Total time of the longest recipe, in minutes.
    pub total_time: u16,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<DishView> for Dish {
    fn from(d: DishView) -> Self {
        Self {
            id: d.id.to_owned(),
            name: d.name.to_owned(),
            recipes: d.recipes.iter().map(Recipe::from).collect(),
            in_list: d.in_list,
            total_time: d.total_time(),
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Dishes {
    pub dishes: Vec<Dish>,
}

/// A recipe the user can put in a dish.
#[derive(Serialize, Clone, Debug)]
pub struct CandidateDto {
    pub id: String,
    pub name: String,
    pub recipe_type: RecipeType,
    pub accepts_accompaniment: bool,
}

impl From<Candidate> for CandidateDto {
    fn from(c: Candidate) -> Self {
        Self {
            id: c.id,
            name: c.name,
            recipe_type: c.recipe_type.0,
            accepts_accompaniment: c.accepts_accompaniment,
        }
    }
}

/// Query of `GET /dishes/candidates`: a page of the pool by name.
#[derive(Deserialize, Debug, Default)]
pub struct CandidatesQuery {
    pub recipe_type: Option<RecipeType>,
    /// Case-insensitive substring of the name.
    pub search: Option<String>,
    /// Page size, 1..=100 (default 20).
    pub limit: Option<u16>,
}

#[derive(Serialize, Debug)]
pub struct Candidates {
    pub candidates: Vec<CandidateDto>,
    /// How many recipes match in all, beyond this page.
    pub total: u64,
}

/// Body of `POST /dishes` and `PUT /dishes/{id}`. `recipe_ids` may come in
/// any order: exactly one main course, at most one appetizer, dessert and
/// beverage, any number of accompaniments and condiments.
#[derive(Deserialize, Debug)]
pub struct ComposeRequest {
    pub name: String,
    pub recipe_ids: Vec<String>,
}

impl From<ComposeRequest> for ComposeInput {
    fn from(r: ComposeRequest) -> Self {
        Self {
            name: r.name,
            recipe_ids: r.recipe_ids,
        }
    }
}
