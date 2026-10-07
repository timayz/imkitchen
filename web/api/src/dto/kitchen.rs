use imkitchen_core::shopping::PoolRecipe;
use imkitchen_types::recipe::{Instruction, RecipeType};
use imkitchen_web_shared::services::kitchen::{
    self, CookingScreen as CookingScreenView, Dish as DishView, IngredientAisle, KitchenList,
    ListEntry, OnboardingMenu as OnboardingMenuView, StepView,
};
use serde::{Deserialize, Serialize};

use super::recipe::{IngredientDto, Recipe, Status, thumbnail_url};

#[derive(Serialize, Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub recipe_type: RecipeType,
    pub status: Status,
    pub advance_prep: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub total_time: u16,
    pub thumbnail_url: Option<String>,
    pub blur_placeholder: Option<String>,
}

impl From<&ListEntry> for Entry {
    fn from(e: &ListEntry) -> Self {
        Self {
            id: e.id.to_owned(),
            name: e.name.to_owned(),
            slug: e.slug.to_owned(),
            recipe_type: e.recipe_type.clone(),
            status: Status::from(&e.status),
            advance_prep: e.advance_prep.to_owned(),
            prep_time: e.prep_time,
            cook_time: e.cook_time,
            total_time: e.total_time(),
            thumbnail_url: thumbnail_url(&e.id, e.thumbnail_version.as_deref(), "mobile"),
            blur_placeholder: e.blur_placeholder.to_owned(),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct StepText {
    pub index: usize,
    pub text: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CurrentStep {
    pub index: usize,
    pub description: String,
    pub time_next: u16,
}

impl CurrentStep {
    fn new((index, instruction): &(usize, Instruction)) -> Self {
        Self {
            index: *index,
            description: instruction.description.to_owned(),
            time_next: instruction.time_next,
        }
    }
}

/// Instructions around the cooking cursor.
#[derive(Serialize, Clone, Debug)]
pub struct Steps {
    pub completed: Vec<StepText>,
    pub coming: Vec<StepText>,
    pub current: Option<CurrentStep>,
}

impl From<&StepView> for Steps {
    fn from((completed, coming, current): &StepView) -> Self {
        let text = |(index, text): &(usize, String)| StepText {
            index: *index,
            text: text.to_owned(),
        };
        Self {
            completed: completed.iter().map(text).collect(),
            coming: coming.iter().map(text).collect(),
            current: current.as_ref().map(CurrentStep::new),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Sample {
    pub id: String,
    pub name: String,
    pub accepts_accompaniment: bool,
}

impl From<&PoolRecipe> for Sample {
    fn from(r: &PoolRecipe) -> Self {
        Self {
            id: r.id.to_owned(),
            name: r.name.to_owned(),
            accepts_accompaniment: r.accepts_accompaniment,
        }
    }
}

/// What the kitchen tab shows, tagged by `kind`.
#[derive(Serialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Overview {
    /// No recipes yet: explain how to add one.
    OnboardingRecipe,
    /// Recipes exist but the list is empty: invite to generate.
    OnboardingMenu {
        recipes: Vec<Sample>,
        main_count: usize,
        appetizer_count: usize,
        accompaniment_count: usize,
        dessert_count: usize,
    },
    List(Box<List>),
}

#[derive(Serialize, Debug)]
pub struct List {
    pub entries: Vec<Entry>,
    pub focused: Option<Recipe>,
    pub focused_status: Status,
    pub completed_count: usize,
    pub total_count: usize,
    pub prep_ahead: Vec<Entry>,
    pub steps: Steps,
    /// "Start cooking" must open `external_url` outside the app.
    pub cook_external: bool,
    pub external_url: Option<String>,
}

impl From<kitchen::Overview> for Overview {
    fn from(view: kitchen::Overview) -> Self {
        match view {
            kitchen::Overview::OnboardingRecipe => Overview::OnboardingRecipe,
            kitchen::Overview::OnboardingMenu(OnboardingMenuView {
                recipes,
                main_count,
                appetizer_count,
                accompaniment_count,
                dessert_count,
            }) => Overview::OnboardingMenu {
                recipes: recipes.iter().map(Sample::from).collect(),
                main_count,
                appetizer_count,
                accompaniment_count,
                dessert_count,
            },
            kitchen::Overview::List(list) => Overview::List(Box::new(List::from(*list))),
        }
    }
}

impl From<KitchenList> for List {
    fn from(list: KitchenList) -> Self {
        let external_url = if list.cook_external {
            list.focused.as_ref().and_then(|r| r.origin.to_owned())
        } else {
            None
        };
        Self {
            entries: list.entries.iter().map(Entry::from).collect(),
            focused: list.focused.as_ref().map(Recipe::from),
            focused_status: Status::from(&list.focused_status),
            completed_count: list.completed_count,
            total_count: list.total_count,
            prep_ahead: list.prep_ahead.iter().map(Entry::from).collect(),
            steps: Steps::from(&list.steps),
            cook_external: list.cook_external,
            external_url,
        }
    }
}

/// The kitchen with another recipe of the list in focus.
#[derive(Serialize, Debug)]
pub struct Dish {
    pub entries: Vec<Entry>,
    pub recipe: Recipe,
    pub status: Status,
    pub steps: Steps,
    pub cook_external: bool,
    pub external_url: Option<String>,
}

impl From<DishView> for Dish {
    fn from(dish: DishView) -> Self {
        let external_url = if dish.cook_external {
            dish.recipe.origin.to_owned()
        } else {
            None
        };
        Self {
            entries: dish.entries.iter().map(Entry::from).collect(),
            recipe: Recipe::from(&dish.recipe),
            status: Status::from(&dish.status),
            steps: Steps::from(&dish.steps),
            cook_external: dish.cook_external,
            external_url,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Aisle {
    /// `shopping_<Category>`, the i18n key the web uses for the aisle label.
    pub key: String,
    pub items: Vec<IngredientDto>,
}

impl From<&IngredientAisle> for Aisle {
    fn from(aisle: &IngredientAisle) -> Self {
        Self {
            key: aisle.name.to_owned(),
            items: aisle.items.iter().map(IngredientDto::from).collect(),
        }
    }
}

/// One screen of the cooking flow.
#[derive(Serialize, Debug)]
pub struct CookingScreen {
    pub recipe: Recipe,
    pub status: Status,
    pub steps: Steps,
    /// The origin page may be shown in a webview in place of in-app steps.
    pub origin_embeddable: bool,
    /// Show the ingredient list (`ingredient_aisles`) instead of a step.
    pub show_ingredients: bool,
    pub ingredient_aisles: Vec<Aisle>,
    /// Nothing to show in-app: open this URL outside the app instead.
    pub external_url: Option<String>,
}

impl From<CookingScreenView> for CookingScreen {
    fn from(screen: CookingScreenView) -> Self {
        let external_url = screen.external_only().map(str::to_owned);
        Self {
            recipe: Recipe::from(&screen.recipe),
            status: Status::from(&screen.status),
            steps: Steps::from(&screen.steps),
            origin_embeddable: screen.origin_embeddable,
            show_ingredients: screen.show_ingredients,
            ingredient_aisles: screen.ingredient_aisles.iter().map(Aisle::from).collect(),
            external_url,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GenerateRequest {
    /// Number of meals to compose (1–30 on the web).
    pub count: u8,
}

#[derive(Deserialize, Debug)]
pub struct StepRequest {
    pub direction: Direction,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Next,
    Prev,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Next => "next",
            Direction::Prev => "prev",
        }
    }
}
