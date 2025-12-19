use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use evento::cursor::{Args, ReadResult};
use imkitchen_recipe::{
    IngredientUnitFormat, RecipeListRow, RecipeRow, RecipeType, RecipesQuery, UserStat,
};

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{NotFoundTemplate, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "recipes-community-detail.html")]
pub struct DetailTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub recipe: RecipeRow,
    pub stat: UserStat,
    pub cook_recipes: ReadResult<RecipeListRow>,
    pub similar_recipes: ReadResult<RecipeListRow>,
}

impl Default for DetailTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            recipe: RecipeRow::default(),
            stat: UserStat::default(),
            cook_recipes: Default::default(),
            similar_recipes: Default::default(),
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

    if recipe.user_id != user.id && !recipe.is_shared {
        return template.render(NotFoundTemplate).into_response();
    }

    let stat =
        crate::try_page_response!(app.recipe_query.find_user_stat(&recipe.user_id), template)
            .unwrap_or_default();

    let exclude_ids = vec![recipe.id.to_owned()];

    let cook_recipes = crate::try_page_response!(
        app.recipe_query.filter(RecipesQuery {
            exclude_ids: Some(exclude_ids),
            user_id: Some(recipe.user_id.to_owned()),
            recipe_type: None,
            cuisine_type: None,
            is_shared: Some(true),
            dietary_restrictions: vec![],
            dietary_where_any: false,
            sort_by: imkitchen_recipe::SortBy::RecentlyAdded,
            args: Args::forward(2, None),
        }),
        template
    );

    let mut exclude_ids = cook_recipes
        .edges
        .iter()
        .map(|n| n.node.id.to_owned())
        .collect::<Vec<_>>();

    exclude_ids.push(recipe.id.to_owned());

    let mut similar_recipes = crate::try_page_response!(
        app.recipe_query.filter(RecipesQuery {
            exclude_ids: Some(exclude_ids.to_vec()),
            user_id: None,
            recipe_type: Some(recipe.recipe_type.0.to_owned()),
            cuisine_type: Some(recipe.cuisine_type.0.to_owned()),
            is_shared: Some(true),
            dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
            dietary_where_any: false,
            sort_by: imkitchen_recipe::SortBy::RecentlyAdded,
            args: Args::forward(6, None),
        }),
        template
    );

    if similar_recipes.edges.len() < 6 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids.to_vec());

        let more_recipes = crate::try_page_response!(
            app.recipe_query.filter(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                cuisine_type: Some(recipe.cuisine_type.0.to_owned()),
                is_shared: Some(true),
                dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
                dietary_where_any: true,
                sort_by: imkitchen_recipe::SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    if similar_recipes.edges.len() < 6 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids.to_vec());

        let more_recipes = crate::try_page_response!(
            app.recipe_query.filter(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                cuisine_type: None,
                is_shared: Some(true),
                dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
                dietary_where_any: false,
                sort_by: imkitchen_recipe::SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    if similar_recipes.edges.len() < 6 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids.to_vec());

        let more_recipes = crate::try_page_response!(
            app.recipe_query.filter(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                cuisine_type: None,
                is_shared: Some(true),
                dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
                dietary_where_any: true,
                sort_by: imkitchen_recipe::SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    if similar_recipes.edges.len() < 6 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids);

        let more_recipes = crate::try_page_response!(
            app.recipe_query.filter(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                cuisine_type: None,
                is_shared: Some(true),
                dietary_restrictions: vec![],
                dietary_where_any: false,
                sort_by: imkitchen_recipe::SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    template
        .render(DetailTemplate {
            user,
            recipe,
            stat,
            cook_recipes,
            similar_recipes,
            ..Default::default()
        })
        .into_response()
}
