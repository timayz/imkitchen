//! Where a meal is picked by hand: `/list/add` puts one on the list, after
//! the others, and `/list/{entry_id}/replace` puts one in the place of
//! another.
//!
//! A meal is one of the dishes or of the recipes of who picks it, cooked
//! for so many people: the page shows the most recent of each, or those a
//! search finds, each with the button that picks it. It works without a
//! script: searching posts the form, which comes back as it was left, and
//! so does the Enter key in a field.
//!
//! What was picked goes to the api as it was picked: what it refuses is
//! said on the page.

use reqwest::Method;
use serde::{Deserialize, Serialize};
use topcoat::{
    Result,
    context::Cx,
    l10n::Translation,
    metadata::Metadata,
    router::{content::Form, error::not_found, href, page},
    view::{View, component, view},
};

use super::{ADDED, Meal, REPLACED, changed, entry_id, meal_of, meals, show};
use crate::app::_signed_in::recipes::_form::Number;
use crate::app::_signed_in::{
    BUTTON_QUIET, FIELD_REFUSED, Failure, H1, H2, INPUT, LABEL, LINK, MAIN, account, asking,
    failed, read, recipes,
};
use crate::app::{api, db, language, t};

/// How many dishes, and how many recipes, the page shows: the others are
/// searched for.
const DISHES: usize = 12;
const RECIPES: usize = 24;

/// What the form posts.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct Typed {
    /// What the dishes and the recipes are searched for.
    q: String,
    /// How many people the meal is cooked for.
    servings: String,
    /// The recipe that was picked, or the dish: neither when the form was
    /// posted to search.
    recipe_id: Option<String>,
    dish_id: Option<String>,
}

/// A meal as the api takes it: the body of `POST /v1/list/entries` and of
/// `PUT /v1/list/entries/{id}`. A number the api cannot read as one is
/// sent as it was typed, for the api to refuse.
#[derive(Serialize)]
struct EntryBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    recipe_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plat_id: Option<&'a str>,
    /// Left out when nothing was typed: as many as the settings say.
    #[serde(skip_serializing_if = "Option::is_none")]
    servings: Option<Number<'a>>,
}

/// How a form that was posted went, when it did not send the browser on.
struct Again {
    typed: Typed,
    failure: Option<Failure>,
}

/// Hands the meal the form picked to the api: one more on the list without
/// `replaces`, in the place of the entry `replaces` with one. Once the api
/// did it and app followed, the browser is sent to the list; anything else
/// is the page again.
async fn hand(cx: &Cx, replaces: Option<&str>, typed: Typed) -> Result<Again> {
    if typed.recipe_id.is_none() && typed.dish_id.is_none() {
        return Ok(Again {
            typed,
            failure: None,
        });
    }
    let body = EntryBody {
        recipe_id: typed.recipe_id.as_deref(),
        plat_id: typed.dish_id.as_deref(),
        servings: Number::whole(&typed.servings),
    };
    let language = language(cx);
    let asking = asking(cx, &language);
    let (done, picked) = match replaces {
        Some(id) => {
            let entry = ["list", "entries", id];
            let replaced = api(cx)
                .command(asking, Method::PUT, &entry, Some(&body))
                .await;
            (REPLACED, replaced)
        }
        None => {
            let added = api(cx)
                .command(asking, Method::POST, &["list", "entries"], Some(&body))
                .await;
            (ADDED, added)
        }
    };
    let failure = changed(cx, done, picked).await?;
    Ok(Again {
        typed,
        failure: Some(failure),
    })
}

#[page("./add", with(Metadata::new().title(t!(pick_add_meta_title))))]
pub async fn add(cx: &Cx) -> Result<impl View> {
    // For as many as the settings say, until another number is typed.
    let settings = read(crate::settings::of(db(cx), &account(cx).id).await)?;
    let typed = Typed {
        servings: settings.servings.to_string(),
        ..Typed::default()
    };

    Ok(view! { picker(again: Again { typed, failure: None }, replaces: None) })
}

#[page(POST "./add", with(Metadata::new().title(t!(pick_add_meta_title))))]
async fn submit_add(cx: &Cx, Form(typed): Form<Typed>) -> Result<impl View> {
    let again = hand(cx, None, typed).await?;

    Ok(view! { picker(again: again, replaces: None) })
}

#[page("./{entry_id}/replace", with(Metadata::new().title(t!(pick_replace_meta_title))))]
pub async fn replace(cx: &Cx) -> Result<impl View> {
    let id = entry_id(cx);
    let entries = read(crate::list::of(db(cx), &account(cx).id).await)?;
    let Some(entry) = entries.iter().find(|entry| entry.id == id) else {
        return Err(not_found().into());
    };
    // For as many as the meal it replaces, until another number is typed.
    let typed = Typed {
        servings: entry.servings.to_string(),
        ..Typed::default()
    };

    Ok(view! {
        picker(again: Again { typed, failure: None }, replaces: Some(id.to_owned()))
    })
}

#[page(
    POST "./{entry_id}/replace",
    with(Metadata::new().title(t!(pick_replace_meta_title)))
)]
async fn submit_replace(cx: &Cx, Form(typed): Form<Typed>) -> Result<impl View> {
    let id = entry_id(cx);
    let again = hand(cx, Some(id), typed).await?;

    Ok(view! { picker(again: again, replaces: Some(id.to_owned())) })
}

/// A dish or a recipe to pick, as the page shows it.
struct Row {
    id: String,
    meal: Meal,
}

/// The page a meal is picked on, as it was left when it comes back: the
/// one that replaces the entry `replaces`, or the one that adds a meal
/// without.
#[component]
async fn picker(cx: &Cx, again: Again, replaces: Option<String>) -> Result<impl View> {
    let Again { typed, failure } = again;
    let owner_id = account(cx).id.as_str();
    let (action, title): (String, Translation<'static>) = match replaces.as_deref() {
        Some(id) => (
            href!(submit_replace, entry_id: id).resolve(cx).to_string(),
            t!(pick_replace_title),
        ),
        None => (
            href!(submit_add).resolve(cx).to_string(),
            t!(pick_add_title),
        ),
    };
    // What the meal takes the place of, when it is still on the list.
    let replaced = match replaces.as_deref() {
        Some(id) => {
            let entries = read(crate::list::of(db(cx), owner_id).await)?;
            match entries.into_iter().find(|entry| entry.id == id) {
                Some(entry) => {
                    let (recipes, dishes) = meals(
                        cx,
                        entry.recipe_id.iter().cloned().collect(),
                        &entry.dish_id.iter().cloned().collect::<Vec<_>>(),
                    )
                    .await?;
                    meal_of(&entry, &recipes, &dishes).map(|meal| meal.title.clone())
                }
                None => None,
            }
        }
        None => None,
    };

    // The most recent, or those the search finds: one more than shown says
    // whether there are others.
    let q = Some(typed.q.trim()).filter(|q| !q.is_empty());
    let found_dishes =
        read(crate::dishes::list_of(db(cx), owner_id, q, None, DISHES as i64 + 1).await)?;
    let found_recipes =
        read(crate::recipes::list_of(db(cx), owner_id, q, None, RECIPES as i64 + 1).await)?;
    let more = found_dishes.len() > DISHES || found_recipes.len() > RECIPES;
    let dish_ids: Vec<String> = found_dishes
        .into_iter()
        .take(DISHES)
        .map(|dish| dish.id)
        .collect();
    let recipe_ids: Vec<String> = found_recipes
        .into_iter()
        .take(RECIPES)
        .map(|recipe| recipe.id)
        .collect();
    let (mut recipes, mut dishes) = meals(cx, recipe_ids.clone(), &dish_ids).await?;
    let rows = |ids: Vec<String>, meals: &mut std::collections::HashMap<String, Meal>| {
        ids.into_iter()
            .filter_map(|id| meals.remove(&id).map(|meal| Row { id, meal }))
            .collect::<Vec<Row>>()
    };
    let dishes = rows(dish_ids, &mut dishes);
    let recipes = rows(recipe_ids, &mut recipes);
    // Nothing to pick, and nothing was searched for: no recipe at all.
    let nothing = dishes.is_empty() && recipes.is_empty() && q.is_none();
    let none_found = dishes.is_empty() && recipes.is_empty();
    let (any_dish, any_recipe) = (!dishes.is_empty(), !recipes.is_empty());

    let refused = |field: &str| failure.as_ref().and_then(|failure| failure.field(field));
    let servings_refused = refused("servings");
    // What the api says of the meal itself: it was deleted meanwhile.
    let meal_refused = refused("recipe_id").or_else(|| refused("plat_id"));
    let row = "flex items-center gap-3 rounded-lg border border-stone-200 bg-white px-3 py-2";
    let thumbnail = "size-10 shrink-0 rounded object-cover";
    let choose = "shrink-0 rounded-lg border border-stone-300 bg-white px-3 py-1 text-sm font-semibold hover:border-stone-500";

    Ok(view! {
        <main class=(MAIN)>
            <p class="text-sm"><a class=(LINK) href=(href!(show))>(t!(list_title))</a></p>
            <h1 class=(H1)>(title)</h1>
            if let Some(replaced) = replaced {
                <p class="mt-4 text-lg text-stone-600" data-replaces="">
                    (t!(pick_replaces, title: replaced.as_str()))
                </p>
            }
            <form class="mt-8 grid gap-5" method="post" action=(action) data-entry=(replaces)>
                // What the Enter key does in a field: a search, which picks
                // nothing. A browser takes the first button of a form.
                <button
                    class="sr-only"
                    type="submit"
                    name="search"
                    value="1"
                    tabindex="-1"
                    aria-hidden="true"
                    formnovalidate=""
                ></button>
                failed(failure: failure)
                if let Some(message) = meal_refused {
                    <p class=(FIELD_REFUSED) data-field="meal">(message)</p>
                }
                <div class="max-w-48">
                    <label class=(LABEL) for="pick-servings">(t!(settings_servings))</label>
                    <input
                        id="pick-servings"
                        class=(INPUT)
                        type="number"
                        name="servings"
                        value=(typed.servings)
                        min="1"
                        inputmode="numeric"
                        aria-invalid=(servings_refused.is_some().then_some("true"))
                    >
                    if let Some(message) = servings_refused {
                        <p class=(FIELD_REFUSED) data-field="servings">(message)</p>
                    }
                </div>
                if nothing {
                    <p class="text-stone-600" data-pick="nothing">
                        (t!(pick_nothing))
                        " "
                        <a class=(LINK) href=(href!(recipes::new::blank))>(t!(recipes_new))</a>
                    </p>
                } else {
                    <div class="flex gap-3">
                        <label class="sr-only" for="pick-q">(t!(pick_search_label))</label>
                        <input
                            id="pick-q"
                            class="block w-full rounded-lg border border-stone-300 bg-white px-3 py-2"
                            type="search"
                            name="q"
                            value=(typed.q)
                            placeholder=(t!(pick_search_label))
                        >
                        <button class=(BUTTON_QUIET) type="submit" name="search" value="1">
                            (t!(recipes_search))
                        </button>
                    </div>
                    if none_found {
                        <p class="text-stone-600" data-pick="none">(t!(pick_none_found))</p>
                    }
                    if any_dish {
                        <fieldset>
                            <legend class=(H2)>(t!(dishes_title))</legend>
                            <ul class="mt-4 grid gap-2" data-rows="dishes">
                                for dish in dishes {
                                    <li class=(row) data-dish=(dish.id.as_str())>
                                        if let Some(picture) = dish.meal.picture {
                                            <img class=(thumbnail) src=(picture) alt="" loading="lazy">
                                        }
                                        <div class="min-w-0 flex-1">
                                            <p class="wrap-break-word">(dish.meal.title.as_str())</p>
                                            if let Some(made_of) = dish.meal.made_of {
                                                <p class="line-clamp-1 text-sm text-stone-500">(made_of)</p>
                                            }
                                        </div>
                                        <button
                                            class=(choose)
                                            type="submit"
                                            name="dish_id"
                                            value=(dish.id.as_str())
                                            aria-label=(t!(pick_choose_of, title: dish.meal.title.as_str()))
                                        >
                                            (t!(pick_choose))
                                        </button>
                                    </li>
                                }
                            </ul>
                        </fieldset>
                    }
                    if any_recipe {
                        <fieldset>
                            <legend class=(H2)>(t!(recipes_title))</legend>
                            <ul class="mt-4 grid gap-2" data-rows="recipes">
                                for recipe in recipes {
                                    <li class=(row) data-recipe=(recipe.id.as_str())>
                                        if let Some(picture) = recipe.meal.picture {
                                            <img class=(thumbnail) src=(picture) alt="" loading="lazy">
                                        }
                                        <p class="min-w-0 flex-1 wrap-break-word">
                                            (recipe.meal.title.as_str())
                                        </p>
                                        <button
                                            class=(choose)
                                            type="submit"
                                            name="recipe_id"
                                            value=(recipe.id.as_str())
                                            aria-label=(t!(pick_choose_of, title: recipe.meal.title.as_str()))
                                        >
                                            (t!(pick_choose))
                                        </button>
                                    </li>
                                }
                            </ul>
                        </fieldset>
                    }
                    if more {
                        <p class="text-sm text-stone-500" data-pick="more">(t!(pick_more))</p>
                    }
                }
            </form>
        </main>
    })
}
