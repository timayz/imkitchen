//! The tutorial: the five steps a new user is walked through, done with
//! what they really do in the application. A recipe written or imported, a
//! dish composed, the settings changed, the list filled and an item of the
//! groceries checked each do their step, wherever they are done from.
//!
//! It is the api's ([`crate::tutorial`] follows it), so it is the same on
//! every device: who leaves finds it where they left it, here or elsewhere.
//! It is shown on the home, every step with where it is done ([`steps`]),
//! and above every other page while a step is ahead, the one the user is
//! at ([`banner`]). It is skipped from the home (`/tutorial/skip`), and
//! skipped or started over from the settings ([`super::settings`]).
//!
//! The api notes a step a moment after the command that does it: a page
//! that hands such a command waits for app to have followed the step too
//! ([`noted`]), so the page the browser is sent to leads to the next one.

use imkitchen_aggregates::tutorial::Step;
use reqwest::Method;
use topcoat::{
    Result,
    context::Cx,
    l10n::Translation,
    router::{error::see_other, href, page},
    view::{View, class, component, view},
};

use super::{
    BUTTON, BUTTON_QUIET, Failure, LINK, account, asking, dishes, failed, followed, groceries,
    home, list, read, recipes, settings, welcome,
};
use crate::api::CommandError;
use crate::app::{api, db, language, t};
use crate::tutorial::{Tutorial, word};

/// What a step asks of the user, and where it is done.
struct Guide {
    title: Translation<'static>,
    text: Translation<'static>,
    /// The pages it is done on, each with what it is done by there: the
    /// first is where the tutorial leads.
    ways: Vec<(String, Translation<'static>)>,
}

fn guide(cx: &Cx, step: Step) -> Guide {
    match step {
        Step::Recipe => Guide {
            title: t!(tutorial_recipe),
            text: t!(tutorial_recipe_text),
            ways: vec![
                (
                    href!(recipes::new::blank).resolve(cx).to_string(),
                    t!(tutorial_recipe_write),
                ),
                (
                    href!(recipes::import::ask).resolve(cx).to_string(),
                    t!(tutorial_recipe_import),
                ),
            ],
        },
        Step::Plat => Guide {
            title: t!(tutorial_plat),
            text: t!(tutorial_plat_text),
            ways: vec![(
                href!(dishes::new::blank).resolve(cx).to_string(),
                t!(tutorial_plat_compose),
            )],
        },
        Step::Settings => Guide {
            title: t!(tutorial_settings),
            text: t!(tutorial_settings_text),
            ways: vec![(
                href!(settings::show).resolve(cx).to_string(),
                t!(tutorial_settings_open),
            )],
        },
        Step::List => Guide {
            title: t!(tutorial_list),
            text: t!(tutorial_list_text),
            ways: vec![
                (
                    href!(list::show).resolve(cx).to_string(),
                    t!(tutorial_list_generate),
                ),
                (
                    href!(list::_pick::add).resolve(cx).to_string(),
                    t!(tutorial_list_pick),
                ),
            ],
        },
        Step::Groceries => Guide {
            title: t!(tutorial_groceries),
            text: t!(tutorial_groceries_text),
            ways: vec![(
                href!(groceries::show).resolve(cx).to_string(),
                t!(tutorial_groceries_open),
            )],
        },
    }
}

/// Where `step` is among the steps, from 1.
pub(crate) fn number(step: Step) -> usize {
    Step::ALL
        .iter()
        .position(|of| *of == step)
        .unwrap_or_default()
        + 1
}

/// Waits until app followed `step`, which a command of who reads the page
/// just did, so that the page the browser is sent to shows the tutorial
/// past it. Nothing is waited for once they skipped the tutorial or did
/// the step before: the api notes nothing then. Nor for long
/// ([`followed`]).
///
/// The command must be one the api wrote something for: one that left
/// things as they stood does no step.
pub(crate) async fn noted(cx: &Cx, step: Step) {
    let (db, user_id) = (db(cx), account(cx).id.as_str());
    followed(|| async {
        let tutorial = crate::tutorial::of(db, user_id).await?;
        Ok(tutorial.skipped || tutorial.did(step))
    })
    .await;
}

/// Skips the tutorial of who reads the page, on every device, and waits
/// until app followed: the page the browser is sent to shows it skipped.
/// The steps they did stay done.
pub(crate) async fn skipped(cx: &Cx) -> Result<(), CommandError> {
    asked(cx, "skip", |tutorial| tutorial.skipped).await
}

/// Starts the tutorial of who reads the page over, on every device: no
/// step is done, and it is not skipped. Waits until app followed.
pub(crate) async fn restarted(cx: &Cx) -> Result<(), CommandError> {
    asked(cx, "restart", |tutorial| *tutorial == Tutorial::default()).await
}

/// Hands the api `command` of the tutorial, then waits until the tutorial,
/// as app followed it, is `shown`.
async fn asked(
    cx: &Cx,
    command: &str,
    shown: impl Fn(&Tutorial) -> bool,
) -> Result<(), CommandError> {
    let language = language(cx);
    api(cx)
        .command(
            asking(cx, &language),
            Method::POST,
            &["tutorial", command],
            None::<&()>,
        )
        .await?;
    let (db, user_id) = (db(cx), account(cx).id.as_str());
    followed(|| async { Ok(shown(&crate::tutorial::of(db, user_id).await?)) }).await;
    Ok(())
}

/// Skips the tutorial from the home, or closes it once every step is done,
/// and sends the browser back to the home; or says why it was not.
#[page(POST "./skip")]
pub(crate) async fn skip(cx: &Cx) -> Result<impl View> {
    let failure = match skipped(cx).await {
        Ok(()) => return Err(see_other(href!(home).resolve(cx)).into()),
        Err(err) => Failure::of(cx, err)?,
    };

    Ok(view! { welcome(failure: Some(failure)) })
}

/// A step of the tutorial, as the home shows it.
struct Row {
    word: &'static str,
    number: usize,
    done: bool,
    /// The step the user is at.
    current: bool,
    guide: Guide,
}

/// The tutorial of who reads the page, on the home: every step, those they
/// did and the one they are at, with where each is done; and what
/// `failure` says of skipping it, when that was just refused. Once every
/// step is done it says so, until it is closed. Nothing once it is skipped.
#[component]
pub(crate) async fn steps(cx: &Cx, failure: Option<Failure>) -> Result<impl View> {
    let tutorial = read(crate::tutorial::of(db(cx), &account(cx).id).await)?;
    let shown = !tutorial.skipped;
    let completed = tutorial.completed();
    let current = tutorial.current();
    let rows: Vec<Row> = Step::ALL
        .into_iter()
        .map(|step| Row {
            word: word(step),
            number: number(step),
            done: tutorial.did(step),
            current: current == Some(step),
            guide: guide(cx, step),
        })
        .collect();
    let (title, text, leave) = if completed {
        (
            t!(tutorial_done_title),
            t!(tutorial_done_text),
            t!(tutorial_close),
        )
    } else {
        (t!(tutorial_title), t!(tutorial_text), t!(tutorial_skip))
    };

    Ok(view! {
        if shown {
            <section
                class="mt-10 rounded-xl border border-stone-200 bg-white p-5 md:p-6"
                aria-labelledby="tutorial-title"
                data-tutorial=(current.map_or("none", word))
            >
                <h2 id="tutorial-title" class="text-xl font-semibold tracking-tight">(title)</h2>
                <p class="mt-1 text-sm text-stone-500">(text)</p>
                <div class="mt-4">failed(failure: failure)</div>
                <ol class="mt-6 grid gap-6">
                    for row in rows {
                        <li
                            class="flex gap-4"
                            data-tutorial-step=(row.word)
                            data-completed=(row.done.to_string())
                            aria-current=(row.current.then_some("step"))
                        >
                            <span
                                class=(class!(
                                    "flex size-8 shrink-0 items-center justify-center rounded-full border text-sm font-semibold tabular-nums",
                                    "border-stone-900 bg-stone-900 text-stone-50" if row.done,
                                    "border-stone-900" if row.current,
                                    "border-stone-300 text-stone-500" if !row.done && !row.current,
                                ))
                                aria-hidden="true"
                            >
                                if row.done {
                                    "✓"
                                } else {
                                    (row.number)
                                }
                            </span>
                            // Its title on the line of the number.
                            <div class="min-w-0 flex-1 pt-1">
                                <h3 class=(class!("font-semibold", "text-stone-500" if row.done))>
                                    (row.guide.title)
                                    if row.done {
                                        <span class="sr-only">" — " (t!(tutorial_step_done))</span>
                                    }
                                </h3>
                                if !row.done {
                                    <p class="mt-1 text-sm text-stone-600">(row.guide.text)</p>
                                    // Buttons for the step the user is at, links
                                    // for those after it.
                                    <p
                                        class=(class!(
                                            "flex flex-wrap",
                                            "mt-3 gap-3" if row.current,
                                            "mt-2 gap-x-4 gap-y-1 text-sm" if !row.current,
                                        ))
                                    >
                                        for (at, (address, label)) in row.guide.ways.into_iter().enumerate() {
                                            <a
                                                class=(match (row.current, at) {
                                                    (true, 0) => BUTTON,
                                                    (true, _) => BUTTON_QUIET,
                                                    (false, _) => LINK,
                                                })
                                                href=(address)
                                            >
                                                (label)
                                            </a>
                                        }
                                    </p>
                                }
                            </div>
                        </li>
                    }
                </ol>
                <form class="mt-8" method="post" action=(href!(skip))>
                    <button class="text-sm font-medium underline hover:text-stone-600" type="submit">
                        (leave)
                    </button>
                </form>
            </section>
        }
    })
}

/// The step of the tutorial who reads the page is at, above every page but
/// the home: what it asks, where it is done, and the way to every step.
/// Nothing once the tutorial is skipped or every step done.
#[component]
pub(crate) async fn banner(cx: &Cx) -> Result<impl View> {
    let tutorial = read(crate::tutorial::of(db(cx), &account(cx).id).await)?;
    let current = tutorial
        .current()
        .map(|step| (word(step), number(step), guide(cx, step)));
    let total = Step::ALL.len();
    let all = href!(home).resolve(cx).to_string();

    Ok(view! {
        if let Some((at, position, asked)) = current {
            <aside
                class="border-b border-stone-200 bg-white"
                aria-label=(t!(tutorial_title))
                data-tutorial-banner=(at)
            >
                // The links beside what the step asks where there is room,
                // under it where there is not.
                <div class="mx-auto flex max-w-3xl flex-wrap items-center gap-x-6 gap-y-1 px-4 py-3 text-sm md:px-8">
                    <p class="grow basis-64">
                        <span class="font-semibold">(t!(tutorial_step_of, number: position, total: total))</span>
                        " · "
                        (asked.title)
                    </p>
                    <p class="flex flex-wrap gap-x-4 gap-y-1">
                        for (address, label) in asked.ways {
                            <a class="font-semibold underline hover:text-stone-600" href=(address)>(label)</a>
                        }
                        <a class="text-stone-500 hover:text-stone-900 hover:underline" href=(all)>
                            (t!(tutorial_all_steps))
                        </a>
                    </p>
                </div>
            </aside>
        }
    })
}
