//! The form a recipe is written with, at `/recipes/new` ([`super::new`])
//! and at `/recipes/{recipe_id}/edit`, where one is rewritten: imported or
//! written, a recipe is rewritten the same way.
//!
//! The form posts everything a recipe says, as a whole, and its picture
//! with it when one is chosen (`multipart/form-data`). An ingredient and a
//! step are a row each; the rows left blank are not part of the recipe, and
//! the form always shows a few more of them than the recipe holds. It works
//! without a script: asking for more rows posts the form, which comes back
//! as it was typed, longer. A picture that was chosen is not part of what
//! comes back, a browser sends a file once: it is chosen last.
//!
//! What was typed goes to the api as it was typed. A number the api cannot
//! read as one is sent as the text it is, for the api to refuse: app
//! decides nothing of what a recipe may hold. What the api refuses of a
//! field is said under the field, or under the row for the fields of an
//! ingredient and of a step.
//!
//! The picture is a command of its own, sent once the recipe is written.
//! When the api refuses it (not a picture, too large), the recipe stays
//! written without it, and the form says both.
//!
//! A recipe written is the first step of the tutorial ([`tutorial`]).

use imkitchen_aggregates::tutorial::Step;
use reqwest::Method;
use serde::Serialize;
use timada_core::Rendition;
use topcoat::{
    Result,
    context::Cx,
    l10n::Translation,
    metadata::Metadata,
    router::{
        BodyLimit,
        content::multipart::Multipart,
        error::{not_found, see_other},
        href, page,
    },
    view::{View, class, component, view},
};

use super::{Written, list, own, recipe_id, show};
use crate::app::_signed_in::{
    BUTTON, BUTTON_QUIET, FIELD_REFUSED, Failure, H1, H2, INPUT, LABEL, LINK, MAIN, asking, failed,
    tutorial,
};
use crate::app::{api, language, t};
use crate::recipes::Recipe;

/// The most a form may weigh: the 8 MiB the api takes of a picture, and
/// room for what the recipe says. A picture between the two is the api's
/// to refuse, in its words.
pub(crate) const FORM_LIMIT: BodyLimit = BodyLimit::max(10 * 1024 * 1024);

/// The kinds of picture the api takes, for the browser to offer no other.
const PICTURES: &str = "image/jpeg,image/png,image/webp,image/gif";

/// How many blank rows the form shows beyond those that are filled, and
/// how many rows it shows at least: of ingredients, then of steps.
const BLANK_ROWS: (usize, usize) = (3, 2);
const LEAST_ROWS: (usize, usize) = (6, 3);
/// How many rows asking for more adds.
const MORE_ROWS: (usize, usize) = (6, 3);

/// An ingredient as it is typed.
#[derive(Debug, Default)]
struct TypedIngredient {
    quantity: String,
    unit: String,
    name: String,
}

impl TypedIngredient {
    fn is_blank(&self) -> bool {
        [&self.quantity, &self.unit, &self.name]
            .iter()
            .all(|text| text.trim().is_empty())
    }
}

/// A step as it is typed.
#[derive(Debug, Default)]
struct TypedStep {
    text: String,
    timer_minutes: String,
}

impl TypedStep {
    fn is_blank(&self) -> bool {
        self.text.trim().is_empty() && self.timer_minutes.trim().is_empty()
    }
}

/// A picture as the browser sent it.
struct Upload {
    content_type: String,
    bytes: Vec<u8>,
}

/// Everything the form holds, as it was typed.
#[derive(Default)]
pub(crate) struct Typed {
    title: String,
    description: String,
    prep_minutes: String,
    cook_minutes: String,
    servings: String,
    /// Without the rows left blank.
    ingredients: Vec<TypedIngredient>,
    /// Without the rows left blank.
    instructions: Vec<TypedStep>,
    picture: Option<Upload>,
    /// Whether the picture the recipe shows is to be taken off.
    remove_picture: bool,
    /// Whether more rows were asked for, and nothing else.
    more: bool,
}

/// A number as it is sent to the api: the one that was typed, or what was
/// typed when it is not one.
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Number<'a> {
    Whole(u64),
    Real(f64),
    Typed(&'a str),
}

impl<'a> Number<'a> {
    /// Minutes, people: nothing when nothing was typed.
    pub(crate) fn whole(typed: &'a str) -> Option<Self> {
        let typed = typed.trim();
        if typed.is_empty() {
            return None;
        }
        Some(typed.parse().map_or(Self::Typed(typed), Self::Whole))
    }

    /// A quantity, its decimals after a point or a comma.
    fn real(typed: &'a str) -> Option<Self> {
        let typed = typed.trim();
        if typed.is_empty() {
            return None;
        }
        let read = typed.replace(',', ".").parse::<f64>().ok();
        Some(
            read.filter(|quantity| quantity.is_finite())
                .map_or(Self::Typed(typed), Self::Real),
        )
    }
}

#[derive(Serialize)]
struct IngredientBody<'a> {
    name: &'a str,
    quantity: Option<Number<'a>>,
    unit: &'a str,
}

#[derive(Serialize)]
struct StepBody<'a> {
    text: &'a str,
    timer_minutes: Option<Number<'a>>,
}

/// What writing a recipe and rewriting one both take: the body of
/// `POST /v1/recipes` and of `PUT /v1/recipes/{id}`.
#[derive(Serialize)]
struct RecipeBody<'a> {
    title: &'a str,
    description: &'a str,
    ingredients: Vec<IngredientBody<'a>>,
    instructions: Vec<StepBody<'a>>,
    prep_minutes: Option<Number<'a>>,
    cook_minutes: Option<Number<'a>>,
    servings: Option<Number<'a>>,
}

impl Typed {
    /// The form of a recipe that is not written yet: for four, as the api
    /// takes an imported one to be.
    pub(crate) fn empty() -> Self {
        Self {
            servings: "4".to_owned(),
            ..Self::default()
        }
    }

    /// The form of `recipe`, as it stands.
    fn of(recipe: &Recipe) -> Self {
        // No time is no time, not zero minutes.
        let minutes = |minutes: i64| Some(minutes).filter(|minutes| *minutes > 0);
        let typed = |number: Option<String>| number.unwrap_or_default();
        Self {
            title: recipe.title.clone(),
            description: recipe.description.clone(),
            prep_minutes: typed(minutes(recipe.prep_minutes).map(|n| n.to_string())),
            cook_minutes: typed(minutes(recipe.cook_minutes).map(|n| n.to_string())),
            servings: recipe.servings.to_string(),
            ingredients: recipe
                .ingredients
                .0
                .iter()
                .map(|one| TypedIngredient {
                    quantity: typed(one.quantity.map(|quantity| quantity.to_string())),
                    unit: one.unit.clone().unwrap_or_default(),
                    name: one.name.clone(),
                })
                .collect(),
            instructions: recipe
                .instructions
                .0
                .iter()
                .map(|one| TypedStep {
                    text: one.text.clone(),
                    timer_minutes: typed(one.timer_minutes.map(|minutes| minutes.to_string())),
                })
                .collect(),
            ..Self::default()
        }
    }

    /// What the form posted. The fields of a row come in the order the form
    /// has them: the first of a row starts it.
    pub(crate) async fn posted(mut multipart: Multipart) -> Result<Self> {
        let mut typed = Self::default();
        while let Some(field) = multipart.next_field().await? {
            let Some(name) = field.name().map(str::to_owned) else {
                continue;
            };
            if name == "picture" {
                let content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_owned();
                let bytes = field.bytes().await?.to_vec();
                // A form whose picture was not chosen sends an empty one.
                if !bytes.is_empty() {
                    typed.picture = Some(Upload {
                        content_type,
                        bytes,
                    });
                }
                continue;
            }
            let text = field.text().await?;
            match name.as_str() {
                "title" => typed.title = text,
                "description" => typed.description = text,
                "prep_minutes" => typed.prep_minutes = text,
                "cook_minutes" => typed.cook_minutes = text,
                "servings" => typed.servings = text,
                "ingredient_quantity" => typed.ingredients.push(TypedIngredient {
                    quantity: text,
                    ..TypedIngredient::default()
                }),
                "ingredient_unit" => {
                    if let Some(row) = typed.ingredients.last_mut() {
                        row.unit = text;
                    }
                }
                "ingredient_name" => {
                    if let Some(row) = typed.ingredients.last_mut() {
                        row.name = text;
                    }
                }
                "instruction_text" => typed.instructions.push(TypedStep {
                    text,
                    ..TypedStep::default()
                }),
                "instruction_timer_minutes" => {
                    if let Some(row) = typed.instructions.last_mut() {
                        row.timer_minutes = text;
                    }
                }
                "remove_picture" => typed.remove_picture = true,
                "more" => typed.more = true,
                _ => {}
            }
        }
        typed.ingredients.retain(|row| !row.is_blank());
        typed.instructions.retain(|row| !row.is_blank());
        Ok(typed)
    }

    fn body(&self) -> RecipeBody<'_> {
        RecipeBody {
            title: &self.title,
            description: &self.description,
            ingredients: self
                .ingredients
                .iter()
                .map(|row| IngredientBody {
                    name: &row.name,
                    quantity: Number::real(&row.quantity),
                    unit: &row.unit,
                })
                .collect(),
            instructions: self
                .instructions
                .iter()
                .map(|row| StepBody {
                    text: &row.text,
                    timer_minutes: Number::whole(&row.timer_minutes),
                })
                .collect(),
            prep_minutes: Number::whole(&self.prep_minutes),
            cook_minutes: Number::whole(&self.cook_minutes),
            servings: Number::whole(&self.servings),
        }
    }
}

/// How a form that was posted went, when it did not send the browser on.
pub(crate) struct Again {
    typed: Typed,
    failure: Option<Failure>,
    /// The recipe the form now rewrites, when the api wrote it and refused
    /// its picture alone.
    written: Option<String>,
}

impl Again {
    /// A form nothing was posted of yet.
    pub(crate) fn of(typed: Typed) -> Self {
        Self {
            typed,
            failure: None,
            written: None,
        }
    }
}

/// Hands what the form posted to the api: a recipe to write without `id`,
/// the recipe `id` to rewrite with one. Once the api did it, and noted the
/// step of the tutorial when the recipe is a new one, the browser is sent
/// to the recipe, which shows what the api did; anything else is the form
/// again.
pub(crate) async fn hand(cx: &Cx, id: Option<&str>, multipart: Multipart) -> Result<Again> {
    let written_anew = id.is_none();
    let mut typed = Typed::posted(multipart).await?;
    let again = |typed, failure, written| {
        Ok(Again {
            typed,
            failure,
            written,
        })
    };
    if typed.more {
        return again(typed, None, None);
    }
    let language = language(cx);
    let asking = asking(cx, &language);
    let written = match id {
        Some(id) => api(cx)
            .command(asking, Method::PUT, &["recipes", id], Some(&typed.body()))
            .await
            .map(|()| id.to_owned()),
        None => api(cx)
            .create::<_, Written>(asking, &["recipes"], &typed.body())
            .await
            .map(|written| written.id),
    };
    let id = match written {
        Ok(id) => id,
        Err(err) => {
            let failure = Failure::of(cx, err)?;
            return again(typed, Some(failure), None);
        }
    };
    let image = ["recipes", id.as_str(), "image"];
    let pictured = match typed.picture.take() {
        Some(upload) => {
            api(cx)
                .upload(asking, &image, &upload.content_type, upload.bytes)
                .await
        }
        None if typed.remove_picture => {
            api(cx)
                .command(asking, Method::DELETE, &image, None::<&()>)
                .await
        }
        None => Ok(()),
    };
    if let Err(err) = pictured {
        let failure = Failure::of(cx, err)?;
        return again(typed, Some(failure), Some(id));
    }
    if written_anew {
        tutorial::noted(cx, Step::Recipe).await;
    }
    Err(see_other(href!(show, recipe_id: id.as_str()).resolve(cx)).into())
}

#[page("./{recipe_id}/edit", with(Metadata::new().title(t!(recipe_edit_meta_title))))]
pub async fn edit(cx: &Cx) -> Result<impl View> {
    let id = recipe_id(cx);
    let Some(recipe) = own(cx, id).await? else {
        return Err(not_found().into());
    };

    Ok(view! {
        form(again: Again::of(Typed::of(&recipe)), rewrites: Some(id.to_owned()))
    })
}

#[page(
    POST "./{recipe_id}/edit",
    with(Metadata::new().title(t!(recipe_edit_meta_title))),
    with(FORM_LIMIT)
)]
async fn submit(cx: &Cx, multipart: Multipart) -> Result<impl View> {
    let id = recipe_id(cx);
    let again = hand(cx, Some(id), multipart).await?;

    Ok(view! { form(again: again, rewrites: Some(id.to_owned())) })
}

/// A row of ingredients as the form shows it.
struct IngredientRow {
    typed: TypedIngredient,
    /// What the api refused of it.
    refused: Vec<String>,
}

/// A row of steps as the form shows it.
struct StepRow {
    typed: TypedStep,
    /// What the api refused of it.
    refused: Vec<String>,
}

/// What the api refused of the row `at` of `rows`, of each of its `fields`.
fn row_refused(failure: Option<&Failure>, rows: &str, at: usize, fields: &[&str]) -> Vec<String> {
    let Some(failure) = failure else {
        return Vec::new();
    };
    fields
        .iter()
        .filter_map(|field| failure.field(&format!("{rows}[{at}].{field}")))
        .collect()
}

/// How many rows a list of `filled` rows is shown with, more of them when
/// `more` were asked for: `pick` takes the count of the list out of those
/// of the two.
fn rows(filled: usize, more: bool, pick: fn((usize, usize)) -> usize) -> usize {
    let rows = (filled + pick(BLANK_ROWS)).max(pick(LEAST_ROWS));
    if more { rows + pick(MORE_ROWS) } else { rows }
}

/// The form, as it was typed when it comes back: the one that rewrites the
/// recipe `rewrites`, or the one that writes a recipe without.
#[component]
pub(crate) async fn form(cx: &Cx, again: Again, rewrites: Option<String>) -> Result<impl View> {
    let Again {
        typed,
        failure,
        written,
    } = again;
    // A recipe the api just wrote without its picture is rewritten from
    // here on.
    let saved_without_picture = written.is_some();
    let rewrites = written.or(rewrites);
    let recipe = match rewrites.as_deref() {
        Some(id) => own(cx, id).await?,
        None => None,
    };
    let shown = recipe
        .as_ref()
        .and_then(|recipe| recipe.file_id.as_deref())
        .map(|file_id| api(cx).picture(file_id, Rendition::Listing));
    let (action, back, title): (String, String, Translation<'static>) = match rewrites.as_deref() {
        Some(id) => (
            href!(submit, recipe_id: id).resolve(cx).to_string(),
            href!(show, recipe_id: id).resolve(cx).to_string(),
            t!(recipe_edit_title),
        ),
        None => (
            href!(super::new::submit).resolve(cx).to_string(),
            href!(list).resolve(cx).to_string(),
            t!(recipe_new_title),
        ),
    };

    let refused = |field: &str| failure.as_ref().and_then(|failure| failure.field(field));
    let title_refused = refused("title");
    let description_refused = refused("description");
    let prep_refused = refused("prep_minutes");
    let cook_refused = refused("cook_minutes");
    let servings_refused = refused("servings");
    let ingredients_refused = refused("ingredients");
    let instructions_refused = refused("instructions");

    let ingredient_rows = rows(typed.ingredients.len(), typed.more, |counts| counts.0);
    let mut ingredients: Vec<IngredientRow> = typed
        .ingredients
        .into_iter()
        .enumerate()
        .map(|(at, typed)| IngredientRow {
            typed,
            refused: row_refused(
                failure.as_ref(),
                "ingredients",
                at,
                &["name", "quantity", "unit"],
            ),
        })
        .collect();
    ingredients.resize_with(ingredient_rows, || IngredientRow {
        typed: TypedIngredient::default(),
        refused: Vec::new(),
    });
    let step_rows = rows(typed.instructions.len(), typed.more, |counts| counts.1);
    let mut steps: Vec<StepRow> = typed
        .instructions
        .into_iter()
        .enumerate()
        .map(|(at, typed)| StepRow {
            typed,
            refused: row_refused(
                failure.as_ref(),
                "instructions",
                at,
                &["text", "timer_minutes"],
            ),
        })
        .collect();
    steps.resize_with(step_rows, || StepRow {
        typed: TypedStep::default(),
        refused: Vec::new(),
    });
    let invalid = |refused: &Option<String>| refused.is_some().then_some("true");
    let row_input = "block w-full rounded-lg border border-stone-300 bg-white px-3 py-2";

    Ok(view! {
        <main class=(MAIN)>
            <p class="text-sm"><a class=(LINK) href=(back.as_str())>(t!(recipe_back))</a></p>
            <h1 class=(H1)>(title)</h1>
            <form
                class="mt-8 grid gap-5"
                method="post"
                action=(action)
                enctype="multipart/form-data"
                data-recipe=(rewrites)
            >
                if saved_without_picture {
                    <p
                        class="rounded-lg border border-green-300 bg-green-50 px-4 py-3 text-green-900"
                        role="status"
                        data-saved="without-picture"
                    >
                        (t!(recipe_saved_without_picture))
                    </p>
                }
                failed(failure: failure)
                <div>
                    <label class=(LABEL) for="recipe-title">(t!(recipe_title))</label>
                    <input
                        id="recipe-title"
                        class=(INPUT)
                        type="text"
                        name="title"
                        value=(typed.title)
                        aria-invalid=(invalid(&title_refused))
                        required=""
                    >
                    if let Some(message) = title_refused {
                        <p class=(FIELD_REFUSED) data-field="title">(message)</p>
                    }
                </div>
                <div>
                    <label class=(LABEL) for="recipe-description">(t!(recipe_description))</label>
                    <textarea
                        id="recipe-description"
                        class=(INPUT)
                        name="description"
                        rows="3"
                        aria-invalid=(invalid(&description_refused))
                    >
                        (typed.description)
                    </textarea>
                    if let Some(message) = description_refused {
                        <p class=(FIELD_REFUSED) data-field="description">(message)</p>
                    }
                </div>
                <div class="grid grid-cols-3 gap-3">
                    <div>
                        <label class=(LABEL) for="recipe-prep">(t!(recipe_prep_minutes))</label>
                        <input
                            id="recipe-prep"
                            class=(INPUT)
                            type="number"
                            name="prep_minutes"
                            value=(typed.prep_minutes)
                            min="0"
                            inputmode="numeric"
                            aria-invalid=(invalid(&prep_refused))
                        >
                        if let Some(message) = prep_refused {
                            <p class=(FIELD_REFUSED) data-field="prep_minutes">(message)</p>
                        }
                    </div>
                    <div>
                        <label class=(LABEL) for="recipe-cook">(t!(recipe_cook_minutes))</label>
                        <input
                            id="recipe-cook"
                            class=(INPUT)
                            type="number"
                            name="cook_minutes"
                            value=(typed.cook_minutes)
                            min="0"
                            inputmode="numeric"
                            aria-invalid=(invalid(&cook_refused))
                        >
                        if let Some(message) = cook_refused {
                            <p class=(FIELD_REFUSED) data-field="cook_minutes">(message)</p>
                        }
                    </div>
                    <div>
                        <label class=(LABEL) for="recipe-servings">(t!(recipe_servings))</label>
                        <input
                            id="recipe-servings"
                            class=(INPUT)
                            type="number"
                            name="servings"
                            value=(typed.servings)
                            min="1"
                            inputmode="numeric"
                            aria-invalid=(invalid(&servings_refused))
                            required=""
                        >
                        if let Some(message) = servings_refused {
                            <p class=(FIELD_REFUSED) data-field="servings">(message)</p>
                        }
                    </div>
                </div>

                <fieldset>
                    <legend class=(H2)>(t!(recipe_ingredients))</legend>
                    <p class="mt-1 text-sm text-stone-500">(t!(recipe_ingredients_hint))</p>
                    if let Some(message) = ingredients_refused {
                        <p class=(FIELD_REFUSED) data-field="ingredients">(message)</p>
                    }
                    <ol class="mt-4 grid gap-2" data-rows="ingredients">
                        for (at, row) in ingredients.into_iter().enumerate() {
                            <li>
                                <div class="grid grid-cols-[4.5rem_4.5rem_minmax(0,1fr)] gap-2">
                                    <input
                                        class=(row_input)
                                        type="text"
                                        name="ingredient_quantity"
                                        value=(row.typed.quantity)
                                        inputmode="decimal"
                                        placeholder=(t!(recipe_ingredient_quantity))
                                        aria-label=(t!(recipe_ingredient_quantity))
                                    >
                                    <input
                                        class=(row_input)
                                        type="text"
                                        name="ingredient_unit"
                                        value=(row.typed.unit)
                                        placeholder=(t!(recipe_ingredient_unit))
                                        aria-label=(t!(recipe_ingredient_unit))
                                    >
                                    <input
                                        class=(row_input)
                                        type="text"
                                        name="ingredient_name"
                                        value=(row.typed.name)
                                        placeholder=(t!(recipe_ingredient_name))
                                        aria-label=(t!(recipe_ingredient_name))
                                    >
                                </div>
                                for message in row.refused {
                                    <p class=(FIELD_REFUSED) data-field=(format!("ingredients[{at}]"))>
                                        (message)
                                    </p>
                                }
                            </li>
                        }
                    </ol>
                </fieldset>

                <fieldset>
                    <legend class=(H2)>(t!(recipe_instructions))</legend>
                    <p class="mt-1 text-sm text-stone-500">(t!(recipe_instructions_hint))</p>
                    if let Some(message) = instructions_refused {
                        <p class=(FIELD_REFUSED) data-field="instructions">(message)</p>
                    }
                    <ol class="mt-4 grid gap-3" data-rows="instructions">
                        for (at, row) in steps.into_iter().enumerate() {
                            <li>
                                <div class="grid grid-cols-[minmax(0,1fr)_5.5rem] gap-2">
                                    <textarea
                                        class=(row_input)
                                        name="instruction_text"
                                        rows="2"
                                        placeholder=(t!(recipe_instruction_text, number: at + 1))
                                        aria-label=(t!(recipe_instruction_text, number: at + 1))
                                    >
                                        (row.typed.text)
                                    </textarea>
                                    <input
                                        class=(class!(row_input, "self-start"))
                                        type="number"
                                        name="instruction_timer_minutes"
                                        value=(row.typed.timer_minutes)
                                        min="1"
                                        inputmode="numeric"
                                        placeholder=(t!(recipe_instruction_timer))
                                        aria-label=(t!(recipe_instruction_timer))
                                    >
                                </div>
                                for message in row.refused {
                                    <p class=(FIELD_REFUSED) data-field=(format!("instructions[{at}]"))>
                                        (message)
                                    </p>
                                }
                            </li>
                        }
                    </ol>
                </fieldset>
                <div>
                    // Nothing has to be filled to ask for rows.
                    <button
                        class=(class!(BUTTON_QUIET, "text-sm"))
                        type="submit"
                        name="more"
                        value="rows"
                        formnovalidate=""
                    >
                        (t!(recipe_more_rows))
                    </button>
                </div>

                <fieldset>
                    <legend class=(H2)>(t!(recipe_picture))</legend>
                    if let Some(shown) = shown {
                        <img
                            class="mt-4 max-h-48 rounded-lg"
                            src=(shown)
                            alt=""
                            loading="lazy"
                            data-picture=""
                        >
                        <label class="mt-3 flex items-center gap-2 text-sm">
                            <input type="checkbox" name="remove_picture" value="1">
                            (t!(recipe_picture_remove))
                        </label>
                    }
                    <label class=(class!(LABEL, "mt-4 block")) for="recipe-picture">
                        (t!(recipe_picture_choose))
                    </label>
                    <input
                        id="recipe-picture"
                        class="mt-1 block w-full text-sm"
                        type="file"
                        name="picture"
                        accept=(PICTURES)
                    >
                    <p class="mt-1 text-sm text-stone-500">(t!(recipe_picture_hint))</p>
                </fieldset>

                <div class="flex flex-wrap items-center gap-4 border-t border-stone-200 pt-6">
                    <button class=(BUTTON) type="submit">(t!(recipe_save))</button>
                    <a class=(LINK) href=(back.as_str())>(t!(recipe_cancel))</a>
                </div>
            </form>
        </main>
    })
}
