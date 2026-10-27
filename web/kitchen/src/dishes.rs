//! Dishes: composed meals the user builds from their library. Listed,
//! composed and edited here; used by list generation and added to the list
//! by hand.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::{Form, Query};
use imkitchen_core::dish::query::CandidateQuery;
use imkitchen_types::recipe::RecipeType;
use serde::Deserialize;

use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::AuthUser;
use imkitchen_web_shared::services::dish::{self, ComposeInput, DishView};
use imkitchen_web_shared::template::{NotFoundTemplate, Template, ToastSuccessTemplate, filters};

#[derive(askama::Template)]
#[template(path = "dishes.html")]
pub struct DishesTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub dishes: Vec<DishView>,
    /// The library has at least one main course: composing is possible.
    pub can_compose: bool,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let dishes = imkitchen_web_shared::try_page_response!(dish::list(&app, &user.id), template);
    let can_compose =
        imkitchen_web_shared::try_page_response!(dish::can_compose(&app, &user.id), template);

    template
        .render(DishesTemplate {
            current_path: "kitchen".to_owned(),
            user,
            dishes,
            can_compose,
        })
        .into_response()
}

/// One recipe the composer offers.
pub struct CourseOption {
    pub id: String,
    pub name: String,
}

/// One course section of the composer: what is picked, pinned on top, and
/// a page of matches for the section's search. Libraries of hundreds of
/// recipes never render whole.
pub struct Course {
    pub recipe_type: RecipeType,
    /// Form field name: `appetizer`, `main_course`, `accompaniments`…
    pub field: &'static str,
    /// Several recipes may be picked (sides and sauces).
    pub multiple: bool,
    /// Translation key of the course name.
    pub label: String,
    /// The section's current search text.
    pub search: String,
    pub selected: Vec<CourseOption>,
    /// Matches not already selected, by name, at most [`CANDIDATE_PAGE`].
    pub results: Vec<CourseOption>,
    /// How many recipes match the search in all (selected ones excluded).
    pub total: u64,
}

impl Course {
    pub fn has_selection(&self) -> bool {
        !self.selected.is_empty()
    }

    /// Matches beyond the page shown.
    pub fn more(&self) -> u64 {
        self.total.saturating_sub(self.results.len() as u64)
    }

    /// Nothing matched the search, not even a pick: say so. A pick that
    /// matches is pinned above and is answer enough.
    pub fn no_match(&self) -> bool {
        let needle = self.search.to_lowercase();
        !needle.is_empty()
            && self.results.is_empty()
            && !self
                .selected
                .iter()
                .any(|o| o.name.to_lowercase().contains(&needle))
    }

    /// The pool has nothing of this course at all (no search, no match).
    pub fn is_empty_pool(&self) -> bool {
        self.search.trim().is_empty() && self.selected.is_empty() && self.total == 0
    }
}

#[derive(askama::Template)]
#[template(path = "dishes-edit.html")]
pub struct EditTemplate {
    pub current_path: String,
    pub user: AuthUser,
    /// `None` when composing a new dish.
    pub id: Option<String>,
    pub name: String,
    pub courses: Vec<Course>,
    /// Why the last submission was refused, re-rendered with its state.
    pub error: Option<String>,
}

/// The option blocks of every course, answered to the live search: each
/// block pushes itself over the one on the page.
#[derive(askama::Template)]
#[template(path = "partials/dishes-options.html")]
pub struct OptionsTemplate {
    pub courses: Vec<Course>,
}

const COURSES: [(RecipeType, &str, bool); 6] = [
    (RecipeType::MainCourse, "main_course", false),
    (RecipeType::Appetizer, "appetizer", false),
    (RecipeType::Accompaniment, "accompaniments", true),
    (RecipeType::Dessert, "dessert", false),
    (RecipeType::Beverage, "beverage", false),
    (RecipeType::Condiment, "condiments", true),
];

/// The composer form. Single courses are radios where `none` means none
/// (explicit, because the minifier drops an empty value and a value-less
/// radio would submit `on`); sides and sauces are checkboxes. The
/// `search_*` fields drive the live search and are ignored on save.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct ComposeForm {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub main_course: String,
    #[serde(default)]
    pub appetizer: String,
    #[serde(default)]
    pub accompaniments: Vec<String>,
    #[serde(default)]
    pub dessert: String,
    #[serde(default)]
    pub beverage: String,
    #[serde(default)]
    pub condiments: Vec<String>,
    #[serde(default)]
    pub search_main_course: String,
    #[serde(default)]
    pub search_appetizer: String,
    #[serde(default)]
    pub search_accompaniments: String,
    #[serde(default)]
    pub search_dessert: String,
    #[serde(default)]
    pub search_beverage: String,
    #[serde(default)]
    pub search_condiments: String,
}

impl ComposeForm {
    /// The form as the editor of an existing dish starts: its name and
    /// recipes, no search.
    fn of(dish: &DishView) -> Self {
        let mut form = ComposeForm {
            name: dish.name.to_owned(),
            ..Default::default()
        };
        for recipe in &dish.recipes {
            match recipe.recipe_type {
                RecipeType::MainCourse => form.main_course = recipe.id.to_owned(),
                RecipeType::Appetizer => form.appetizer = recipe.id.to_owned(),
                RecipeType::Accompaniment => form.accompaniments.push(recipe.id.to_owned()),
                RecipeType::Dessert => form.dessert = recipe.id.to_owned(),
                RecipeType::Beverage => form.beverage = recipe.id.to_owned(),
                RecipeType::Condiment => form.condiments.push(recipe.id.to_owned()),
            }
        }
        form
    }

    fn selected_ids(&self) -> Vec<String> {
        [
            self.appetizer.as_str(),
            self.main_course.as_str(),
            self.dessert.as_str(),
            self.beverage.as_str(),
        ]
        .into_iter()
        .map(str::to_owned)
        .chain(self.accompaniments.iter().cloned())
        .chain(self.condiments.iter().cloned())
        .filter(|id| !id.is_empty() && id != "none")
        .collect()
    }

    fn search(&self, field: &str) -> &str {
        match field {
            "main_course" => &self.search_main_course,
            "appetizer" => &self.search_appetizer,
            "accompaniments" => &self.search_accompaniments,
            "dessert" => &self.search_dessert,
            "beverage" => &self.search_beverage,
            _ => &self.search_condiments,
        }
    }
}

impl From<ComposeForm> for ComposeInput {
    fn from(form: ComposeForm) -> Self {
        ComposeInput {
            recipe_ids: form.selected_ids(),
            name: form.name,
        }
    }
}

/// Build the six course sections from the form state: the picked recipes
/// (resolved against the pool, so ids that left it vanish) pinned on top
/// and a page of matches for each section's search.
async fn courses(app: &AppState, user_id: &str, form: &ComposeForm) -> anyhow::Result<Vec<Course>> {
    let selected = dish::candidates_by_ids(app, user_id, &form.selected_ids()).await?;
    let mut courses = Vec::with_capacity(COURSES.len());
    for (recipe_type, field, multiple) in COURSES.iter() {
        let picked: Vec<CourseOption> = selected
            .iter()
            .filter(|c| &c.recipe_type.0 == recipe_type)
            .map(|c| CourseOption {
                id: c.id.to_owned(),
                name: c.name.to_owned(),
            })
            .collect();
        let search = form.search(field).trim().to_owned();
        let page = dish::candidates(
            app,
            user_id,
            CandidateQuery {
                recipe_type: Some(recipe_type.clone()),
                search: Some(search.to_owned()),
                exclude_ids: picked.iter().map(|o| o.id.to_owned()).collect(),
                limit: dish::CANDIDATE_PAGE,
            },
        )
        .await?;
        courses.push(Course {
            recipe_type: recipe_type.clone(),
            field,
            multiple: *multiple,
            label: recipe_type.to_string(),
            search,
            selected: picked,
            results: page
                .items
                .into_iter()
                .map(|c| CourseOption {
                    id: c.id,
                    name: c.name,
                })
                .collect(),
            total: page.total,
        });
    }
    Ok(courses)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn new_page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let courses = imkitchen_web_shared::try_page_response!(
        courses(&app, &user.id, &ComposeForm::default()),
        template
    );

    template
        .render(EditTemplate {
            current_path: "kitchen".to_owned(),
            user,
            id: None,
            name: String::new(),
            courses,
            error: None,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn edit_page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let Some(dish) =
        imkitchen_web_shared::try_page_response!(dish::find(&app, &user.id, &id), template)
    else {
        return template.render(NotFoundTemplate).into_response();
    };
    let form = ComposeForm::of(&dish);
    let courses =
        imkitchen_web_shared::try_page_response!(courses(&app, &user.id, &form), template);

    template
        .render(EditTemplate {
            current_path: "kitchen".to_owned(),
            user,
            id: Some(dish.id),
            name: dish.name,
            courses,
            error: None,
        })
        .into_response()
}

/// The live search: the form state comes as the query string and every
/// course's option block is answered, pinned picks first.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn options(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Query(form): Query<ComposeForm>,
) -> impl IntoResponse {
    let courses =
        imkitchen_web_shared::try_page_response!(courses(&app, &user.id, &form), template);

    (
        [("ts-swap", "skip")],
        template.render(OptionsTemplate { courses }),
    )
        .into_response()
}

/// A refused composition: the editor again, with the message and the
/// submitted state.
async fn refused(
    template: Template,
    app: &AppState,
    user: AuthUser,
    id: Option<String>,
    form: ComposeForm,
    err: imkitchen_core::Error,
) -> axum::response::Response {
    let message = match err {
        imkitchen_core::Error::User(message) => message,
        imkitchen_core::Error::Validate(_) => "A dish needs a name".to_owned(),
        imkitchen_core::Error::NotFound(_) => {
            return template.render(NotFoundTemplate).into_response();
        }
        err => {
            tracing::error!("{err}");
            return template
                .render(imkitchen_web_shared::template::ServerTemplate)
                .into_response();
        }
    };
    let courses = imkitchen_web_shared::try_page_response!(courses(app, &user.id, &form), template);

    (
        StatusCode::UNPROCESSABLE_ENTITY,
        template.render(EditTemplate {
            current_path: "kitchen".to_owned(),
            user,
            id,
            name: form.name,
            courses,
            error: Some(message),
        }),
    )
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn create_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Form(form): Form<ComposeForm>,
) -> impl IntoResponse {
    match dish::create(&app, &user.id, form.clone().into()).await {
        Ok(_) => Redirect::to("/dishes").into_response(),
        Err(err) => refused(template, &app, user, None, form, err).await,
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn update_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
    Form(form): Form<ComposeForm>,
) -> impl IntoResponse {
    match dish::update(&app, &user.id, &id, form.clone().into()).await {
        Ok(()) => Redirect::to("/dishes").into_response(),
        Err(err) => refused(template, &app, user, Some(id), form, err).await,
    }
}

/// Where the browser goes once a twinspark-submitted form succeeded.
fn go_to(path: &str) -> axum::response::Response {
    ([("ts-location", path.to_owned())], "").into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete_action(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(dish::delete(&app, &user.id, &id), template);

    go_to("/dishes")
}

#[derive(askama::Template)]
#[template(path = "partials/dishes-add-button.html")]
pub struct AddButtonTemplate {
    pub id: String,
    pub in_list: bool,
}

/// Add the dish to the list and swap its button to "In my list".
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn add_to_list_action(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(dish::add_to_list(&app, &user.id, &id), template);

    let button = template.to_string(AddButtonTemplate { id, in_list: true });
    template
        .render(ToastSuccessTemplate {
            original: Some(&button),
            message: "Dish added to your list",
            description: None,
        })
        .into_response()
}

/// Legacy bookmark shape kept symmetrical with the recipe routes.
pub async fn redirect_to_list() -> impl IntoResponse {
    Redirect::to("/dishes")
}

pub fn routes() -> axum::Router<AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/dishes", get(page).post(create_action))
        .route("/dishes/new", get(new_page))
        .route("/dishes/new/options", get(options))
        .route("/dishes/{id}", get(redirect_to_list).post(update_action))
        .route("/dishes/{id}/edit", get(edit_page))
        .route("/dishes/{id}/delete", post(delete_action))
        .route("/dishes/{id}/add-to-list", post(add_to_list_action))
}
