//! `/recipes/{recipe_id}/cook`: the recipe being cooked, in a kitchen: one
//! step at a time, large, with the ingredients at hand.
//!
//! The page holds the whole recipe, and reads as one without a script:
//! every step, in order. Its script (`cook.js`) shows one step at a time,
//! keeps the screen awake where the browser has the Screen Wake Lock, and
//! runs the timers of the steps that have one: started by who cooks,
//! counting down on the page, with a sound and a notification when the
//! browser grants them.
//!
//! The step being read and the timers that were started are kept in the
//! browser (local storage, under the id of the recipe): they survive
//! reloading the page and closing the tab, and are gone once the cooking
//! is finished or stopped. Nothing of them is sent to the api, nor found
//! on another device.
//!
//! `servings` in the address says how many it is cooked for, when that is
//! not as many as the recipe is written for (a meal of the list): the
//! quantities follow.
//!
//! The script says nothing itself: every sentence it shows is in the page,
//! in the language it is read in.

use timada_core::Rendition;
use topcoat::{
    Result,
    asset::{Asset, asset},
    context::Cx,
    metadata::Metadata,
    router::{error::not_found, href, page, query_params},
    view::{View, class, view},
};

use super::{own, recipe_id, show};
use crate::app::_signed_in::{BUTTON, BUTTON_QUIET, H1, H2, LINK};
use crate::app::{api, t};
use crate::recipes::Ingredient;

/// What shows a step at a time, keeps the screen awake and runs the timers.
pub const SCRIPT: Asset = asset!("./cook.js");

/// The most a meal is cooked for, as the api has it.
const MAX_SERVINGS: i64 = 1000;

#[query_params]
struct Query {
    /// How many it is cooked for, when not as many as the recipe says.
    servings: Option<String>,
}

/// `quantity`, written for `written` people, for `cooked`: to two decimals.
fn scaled(quantity: f64, written: i64, cooked: i64) -> f64 {
    if written <= 0 || written == cooked {
        return quantity;
    }
    (quantity * cooked as f64 / written as f64 * 100.0).round() / 100.0
}

/// An ingredient as a recipe writes it, its quantity for `cooked` people
/// when the recipe is written for `written`: "300 g flour", "salt".
fn line(ingredient: &Ingredient, written: i64, cooked: i64) -> String {
    [
        ingredient
            .quantity
            .map(|quantity| scaled(quantity, written, cooked).to_string()),
        ingredient.unit.clone(),
        Some(ingredient.name.clone()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
}

/// A step as the page shows it.
struct Step {
    /// From 0: what the script knows it by.
    at: usize,
    /// From 1: what a person reads.
    number: usize,
    text: String,
    timer_minutes: Option<u32>,
}

#[page("./{recipe_id}/cook", with(Metadata::new().title(t!(cook_meta_title))))]
pub async fn cook(cx: &Cx) -> Result<impl View> {
    let id = recipe_id(cx);
    let Some(recipe) = own(cx, id).await? else {
        return Err(not_found().into());
    };
    let servings = query_params::<Query>(cx)
        .ok()
        .and_then(|query| query.servings.as_deref())
        .and_then(|servings| servings.trim().parse::<i64>().ok())
        .filter(|servings| (1..=MAX_SERVINGS).contains(servings))
        .unwrap_or(recipe.servings);
    let ingredients: Vec<String> = recipe
        .ingredients
        .0
        .iter()
        .map(|ingredient| line(ingredient, recipe.servings, servings))
        .collect();
    // Twice on the page: beside the steps where there is room, folded
    // above them where there is not.
    let at_hand = ingredients.clone();
    let any_ingredient = !ingredients.is_empty();
    let steps: Vec<Step> = recipe
        .instructions
        .0
        .into_iter()
        .enumerate()
        .map(|(at, step)| Step {
            at,
            number: at + 1,
            text: step.text,
            timer_minutes: step.timer_minutes,
        })
        .collect();
    let total = steps.len();
    let picture = recipe
        .file_id
        .as_deref()
        .map(|file_id| api(cx).picture(file_id, Rendition::Listing));
    let back = href!(show, recipe_id: id).resolve(cx).to_string();
    let control = "min-w-32 rounded-lg px-5 py-4 text-lg font-semibold";

    Ok(view! {
        <main
            class="mx-auto max-w-5xl px-4 py-6 md:px-8 md:py-10"
            data-cook=(recipe.id)
            data-title=(recipe.title.as_str())
        >
            <div class="flex flex-wrap items-center justify-between gap-3 text-sm">
                // Stopping forgets where the cooking was.
                <a class=(LINK) href=(back.as_str()) data-cook-leave="">(t!(cook_stop))</a>
                <p class="text-stone-500" data-servings=(servings)>
                    (t!(recipe_serves, count: servings))
                </p>
            </div>
            <div class="mt-3 flex items-center gap-4">
                if let Some(picture) = picture {
                    <img class="size-14 shrink-0 rounded-lg object-cover" src=(picture) alt="">
                }
                <h1 class=(class!(H1, "wrap-break-word"))>(recipe.title.as_str())</h1>
            </div>
            // Said by the script, where the browser cannot keep the screen
            // awake.
            <p class="mt-3 text-sm text-stone-500" data-cook-asleep="" hidden="">
                (t!(cook_asleep))
            </p>
            // The timers that were started, whatever step is being read.
            <div class="mt-4 grid gap-2" data-cook-timers="" aria-live="polite"></div>

            if any_ingredient {
                <details class="mt-6 rounded-xl border border-stone-200 bg-white px-4 py-3 md:hidden">
                    <summary class="cursor-pointer font-semibold">(t!(recipe_ingredients))</summary>
                    <ul class="mt-3 list-disc space-y-1 pl-6">
                        for ingredient in at_hand {
                            <li>(ingredient)</li>
                        }
                    </ul>
                </details>
            }

            <div class="mt-6 md:grid md:grid-cols-[minmax(0,1fr)_18rem] md:gap-10">
                <section>
                    if total == 0 {
                        <p class="text-lg text-stone-600" data-cook-empty="">(t!(cook_no_step))</p>
                    }
                    <ol class="grid gap-8" data-cook-steps=(total)>
                        for step in steps {
                            <li
                                data-cook-step=(step.at)
                                data-label=(t!(cook_step, number: step.number))
                                data-minutes=(step.timer_minutes)
                                data-over=(t!(cook_timer_over_of, number: step.number))
                            >
                                <p class="text-sm font-semibold text-stone-500">
                                    (t!(cook_step_of, number: step.number, total: total))
                                </p>
                                <p class="mt-3 whitespace-pre-line text-2xl leading-relaxed md:text-3xl md:leading-relaxed">
                                    (step.text)
                                </p>
                                if let Some(minutes) = step.timer_minutes {
                                    <p class="mt-5 flex flex-wrap items-center gap-3">
                                        <span class="text-lg text-stone-600" data-timer=(minutes)>
                                            (t!(cook_timer, minutes: minutes))
                                        </span>
                                        // Without the script there is no
                                        // timer to start.
                                        <button
                                            class=(class!(BUTTON_QUIET, "disabled:opacity-30"))
                                            type="button"
                                            data-cook-start=""
                                            hidden=""
                                        >
                                            (t!(cook_timer_start))
                                        </button>
                                    </p>
                                }
                            </li>
                        }
                    </ol>
                    // Shown by the script, which shows one step at a time.
                    <div
                        class="mt-10 flex items-center justify-between gap-3 border-t border-stone-200 pt-6"
                        data-cook-controls=""
                        hidden=""
                    >
                        <button
                            class=(class!(control, "border border-stone-300 bg-white hover:border-stone-500 disabled:opacity-30"))
                            type="button"
                            data-cook-previous=""
                        >
                            (t!(cook_previous))
                        </button>
                        <button
                            class=(class!(control, "bg-stone-900 text-stone-50 hover:bg-stone-700"))
                            type="button"
                            data-cook-next=""
                        >
                            (t!(cook_next))
                        </button>
                        // On the last step: finishing forgets where the
                        // cooking was.
                        <a
                            class=(class!(control, "bg-stone-900 text-center text-stone-50 hover:bg-stone-700"))
                            href=(back.as_str())
                            data-cook-finish=""
                            hidden=""
                        >
                            (t!(cook_finish))
                        </a>
                    </div>
                    <noscript>
                        <p class="mt-10 border-t border-stone-200 pt-6">
                            <a class=(BUTTON) href=(back.as_str())>(t!(cook_finish))</a>
                        </p>
                    </noscript>
                </section>
                if any_ingredient {
                    <aside class="hidden md:block">
                        <h2 class=(class!(H2, "mt-0"))>(t!(recipe_ingredients))</h2>
                        <ul class="mt-4 list-disc space-y-1 pl-6" data-ingredients=(ingredients.len())>
                            for ingredient in ingredients {
                                <li>(ingredient)</li>
                            }
                        </ul>
                    </aside>
                }
            </div>

            // A timer that was started, as the script shows it.
            <template data-cook-timer-row="">
                <div
                    class="flex flex-wrap items-center gap-3 rounded-lg border border-stone-300 bg-white px-4 py-3"
                    data-cook-running=""
                >
                    <button class=(class!(LINK, "text-left")) type="button" data-cook-show=""></button>
                    <span class="text-xl font-semibold tabular-nums" data-cook-left=""></span>
                    <strong class="text-red-800" data-cook-over="" hidden="">(t!(cook_timer_over))</strong>
                    <button
                        class="ml-auto rounded-lg border border-stone-300 bg-white px-3 py-1 text-sm font-semibold hover:border-stone-500"
                        type="button"
                        data-cook-cancel=""
                    >
                        (t!(cook_timer_stop))
                    </button>
                </div>
            </template>
        </main>
        <script src=(SCRIPT) defer=""></script>
    })
}
