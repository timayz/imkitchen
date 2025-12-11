use std::str::FromStr;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use axum_extra::extract::Form;
use imkitchen_recipe::{
    CuisineType, DietaryRestriction, Ingredient, IngredientCategory, IngredientUnit, Instruction,
    RecipeType, UpdateInput,
};
use imkitchen_shared::Metadata;
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{ForbiddenTemplate, Template, ToastSuccessTemplate, filters},
};

#[derive(Deserialize, Default, Clone)]
pub struct EditForm {
    pub recipe_type: RecipeType,
    pub name: String,
    pub description: String,
    pub household_size: u16,
    pub prep_time: u16,
    pub cook_time: u16,
    #[serde(default)]
    pub ingredients: Vec<Ingredient>,
    #[serde(default)]
    pub ingredients_quantity: Vec<u32>,
    #[serde(default)]
    pub ingredients_unit: Vec<String>,
    #[serde(default)]
    pub ingredients_name: Vec<String>,
    #[serde(default)]
    pub ingredients_category: Vec<String>,
    #[serde(default)]
    pub instructions: Vec<Instruction>,
    #[serde(default)]
    pub instructions_description: Vec<String>,
    #[serde(default)]
    pub instructions_time_next: Vec<u16>,
    #[serde(default)]
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_type: CuisineType,
    #[serde(default)]
    pub accepts_accompaniment: String,
    pub advance_prep: String,
}

#[derive(askama::Template)]
#[template(path = "recipes-edit-instruction-row.html")]
pub struct EditInstructionRowTemplate;

#[derive(askama::Template)]
#[template(path = "recipes-edit-ingredient-row.html")]
pub struct EditIngredientRowTemplate;

#[derive(askama::Template)]
#[template(path = "recipes-edit.html")]
pub struct EditTemplate {
    pub id: String,
    pub current_path: String,
    pub user: AuthUser,
    pub form: EditForm,
}

impl Default for EditTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            form: EditForm::default(),
            id: "".to_owned(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let recipe = crate::try_page_response!(opt: app.recipe_query.find(&id), template);

    if recipe.user_id != user.id {
        return template.render(ForbiddenTemplate).into_response();
    }

    let accepts_accompaniment = if recipe.accepts_accompaniment {
        "on"
    } else {
        ""
    };

    template
        .render(EditTemplate {
            user,
            form: EditForm {
                recipe_type: recipe.recipe_type.0,
                name: recipe.name,
                description: recipe.description,
                household_size: recipe.household_size,
                prep_time: recipe.prep_time,
                cook_time: recipe.cook_time,
                ingredients: recipe.ingredients.0,
                instructions: recipe.instructions.0,
                dietary_restrictions: recipe.dietary_restrictions.0,
                cuisine_type: recipe.cuisine_type.0,
                accepts_accompaniment: accepts_accompaniment.to_owned(),
                advance_prep: recipe.advance_prep,
                ingredients_unit: vec![],
                ingredients_name: vec![],
                ingredients_quantity: vec![],
                ingredients_category: vec![],
                instructions_description: vec![],
                instructions_time_next: vec![],
            },
            id,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
    Form(input): Form<EditForm>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt:
        app.recipe_command.load_optional(&id),
        template
    );

    if recipe.item.deleted {
        crate::try_response!(sync: Ok(None::<()>), template);
    }

    if input.ingredients_name.len() != input.ingredients_quantity.len()
        || input.ingredients_name.len() != input.ingredients_unit.len()
        || input.ingredients_name.len() != input.ingredients_category.len()
    {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::Server(
                "ingredients_name, ingredients_quantity, ingredients_unit and ingredients_category size not matched"
                    .to_owned()
            )),
            template
        );
    }

    if input.instructions_description.len() != input.instructions_time_next.len() {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::Server(
                "instructions_description and instructions_time_next size not matched"
                    .to_owned()
            )),
            template
        );
    }

    if recipe.item.user_id != user.id {
        crate::try_response!(sync: Err(imkitchen_shared::Error::Forbidden), template);
    }

    let mut ingredients = vec![];
    for (pos, name) in input.ingredients_name.iter().skip(2).enumerate() {
        ingredients.push(Ingredient {
            name: name.to_owned(),
            unit: IngredientUnit::from_str(&input.ingredients_unit[pos + 2]).ok(),
            category: IngredientCategory::from_str(&input.ingredients_category[pos + 2]).ok(),
            quantity: input.ingredients_quantity[pos + 2].to_owned(),
        });
    }

    let mut instructions = vec![];
    for (pos, description) in input.instructions_description.iter().skip(2).enumerate() {
        instructions.push(Instruction {
            description: description.to_owned(),
            time_next: input.instructions_time_next[pos + 2].to_owned(),
        });
    }

    crate::try_response!(
        app.recipe_command.update(
            UpdateInput {
                id: id.to_owned(),
                recipe_type: input.recipe_type,
                name: input.name,
                description: input.description,
                household_size: input.household_size,
                prep_time: input.prep_time,
                cook_time: input.cook_time,
                ingredients,
                instructions,
                dietary_restrictions: input.dietary_restrictions,
                cuisine_type: input.cuisine_type,
                accepts_accompaniment: input.accepts_accompaniment == "on",
                advance_prep: input.advance_prep,
            },
            &Metadata::by(user.id.to_owned()),
        ),
        template
    );

    template
        .render(ToastSuccessTemplate {
            original: None,
            message: "Recipe saved successfully",
            description: None,
        })
        .into_response()
}

pub async fn ingredient_row(template: Template) -> impl IntoResponse {
    template.render(EditIngredientRowTemplate)
}

pub async fn instruction_row(template: Template) -> impl IntoResponse {
    template.render(EditInstructionRowTemplate)
}
