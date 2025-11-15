use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use axum_extra::extract::Form;
use imkitchen_recipe::{
    AccompanimentType, CuisineType, DietaryRestriction, Ingredient, Instruction, RecipeType,
    UpdateInput,
};
use imkitchen_shared::Metadata;
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    auth::AuthUser,
    query::query_recipe_detail_by_id,
    routes::AppState,
    template::{
        FORBIDDEN, ForbiddenTemplate, NOT_FOUND, NotFoundTemplate, SERVER_ERROR_MESSAGE,
        ServerErrorTemplate, Template, filters,
    },
};

#[derive(Deserialize, Default, Clone)]
pub struct EditForm {
    pub recipe_type: RecipeType,
    pub name: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    #[serde(default)]
    pub ingredients: Vec<Ingredient>,
    #[serde(default)]
    pub instructions: Vec<Instruction>,
    #[serde(default)]
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_type: CuisineType,
    #[serde(default)]
    pub accepts_accompaniment: String,
    #[serde(default)]
    pub preferred_accompaniment_types: Vec<AccompanimentType>,
    pub advance_prep: String,
}

#[derive(askama::Template)]
#[template(path = "recipes-edit.html")]
pub struct EditTemplate {
    pub id: String,
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
    pub form: EditForm,
    pub error_message: Option<String>,
}

impl Default for EditTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            form: EditForm::default(),
            error_message: None,
            id: "".to_owned(),
        }
    }
}

pub async fn page(
    template: Template<EditTemplate>,
    server_error: Template<ServerErrorTemplate>,
    not_found_error: Template<NotFoundTemplate>,
    forbidden_error: Template<ForbiddenTemplate>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let recipe = match query_recipe_detail_by_id(&app.pool, &id).await {
        Ok(Some(r)) => r,
        Ok(_) => return not_found_error.render(NotFoundTemplate).into_response(),
        Err(err) => {
            tracing::error!(recipe = id, user = user.id, err = %err,"Failed to get recipe");

            return server_error.render(ServerErrorTemplate).into_response();
        }
    };

    if recipe.user_id != user.id {
        return forbidden_error.render(ForbiddenTemplate).into_response();
    }

    let accepts_accompaniment = if recipe.accepts_accompaniment {
        "on"
    } else {
        ""
    };

    println!("{:?}", recipe.dietary_restrictions);

    template
        .render(EditTemplate {
            user,
            form: EditForm {
                recipe_type: recipe.recipe_type,
                name: recipe.name,
                description: recipe.description,
                prep_time: recipe.prep_time,
                cook_time: recipe.cook_time,
                ingredients: recipe.ingredients,
                instructions: recipe.instructions,
                dietary_restrictions: recipe.dietary_restrictions,
                cuisine_type: recipe.cuisine_type,
                accepts_accompaniment: accepts_accompaniment.to_owned(),
                preferred_accompaniment_types: recipe.preferred_accompaniment_types,
                advance_prep: recipe.advance_prep,
            },
            id,
            ..Default::default()
        })
        .into_response()
}

pub async fn action(
    template: Template<EditTemplate>,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
    Form(input): Form<EditForm>,
) -> impl IntoResponse {
    let recipe = match app.recipe_command.load_optional(&id).await {
        Ok(Some(loaded)) => loaded,
        Ok(_) => {
            return template.render(EditTemplate {
                id,
                user,
                form: input,
                error_message: Some(NOT_FOUND.to_owned()),
                ..Default::default()
            });
        }
        Err(e) => {
            tracing::error!(recipe = id,user = user.id, err = %e,"Faield to load recipe from action");
            return template.render(EditTemplate {
                id,
                user,
                form: input,
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
                ..Default::default()
            });
        }
    };

    if recipe.item.deleted {
        return template.render(EditTemplate {
            id,
            user,
            form: input,
            error_message: Some(NOT_FOUND.to_owned()),
            ..Default::default()
        });
    }

    if recipe.item.user_id != user.id {
        return template.render(EditTemplate {
            id,
            user,
            form: input,
            error_message: Some(FORBIDDEN.to_owned()),
            ..Default::default()
        });
    }

    let form = input.clone();

    match app
        .recipe_command
        .update(
            UpdateInput {
                id: id.to_owned(),
                recipe_type: input.recipe_type,
                name: input.name,
                description: input.description,
                prep_time: input.prep_time,
                cook_time: input.cook_time,
                ingredients: input.ingredients,
                instructions: input.instructions,
                dietary_restrictions: input.dietary_restrictions,
                cuisine_type: input.cuisine_type,
                accepts_accompaniment: input.accepts_accompaniment == "on",
                preferred_accompaniment_types: input.preferred_accompaniment_types,
                advance_prep: input.advance_prep,
            },
            Metadata::by(user.id.to_owned()),
        )
        .await
    {
        Ok(_) => template.render(EditTemplate {
            id,
            user,
            form,
            ..Default::default()
        }),
        Err(imkitchen_shared::Error::Unknown(e)) => {
            tracing::error!("{e}");

            template.render(EditTemplate {
                id,
                user,
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
                form,
                ..Default::default()
            })
        }
        Err(e) => template.render(EditTemplate {
            id,
            user,
            error_message: Some(e.to_string()),
            form,
            ..Default::default()
        }),
    }
}
