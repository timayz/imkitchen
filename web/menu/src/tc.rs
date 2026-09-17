//! SPIKE: topcoat menu-generate slice, served at `/_tc/menu`.
//!
//! Proves the replacement for the twinspark Pending → Checking polling loop:
//! one `#[procedure]` issues the command and waits server-side for the read
//! model to catch up, then the browser bumps a signal the page reads, so the
//! page re-runs on the server and its content is morphed in place.

use std::time::{Duration, Instant};

use imkitchen_web_shared::tc::{
    auth::require_user,
    context::{i18n, state},
    error::{Outcome, toast_message},
    view::base,
};
use time::OffsetDateTime;
use topcoat::{
    Result,
    context::Cx,
    router::{RouterBuilder, page},
    runtime::{Event, RouterBuilderProcedureExt, procedure, signal},
    view::{View, view},
};

use crate::{generation_caught_up, request_generation};

/// How long one call waits for the read model before giving up. The command
/// is already accepted at that point; the plan shows up on the next load.
const CATCH_UP_TIMEOUT: Duration = Duration::from_secs(15);
const CATCH_UP_INTERVAL: Duration = Duration::from_millis(150);

/// Generates the meal plan for the month of `date` and returns once it is
/// queryable. `Ok(true)` = caught up, `Ok(false)` = accepted but still
/// projecting, `Err(text)` = toast.
///
/// Self-authorizing: page guards do not run for procedure endpoints. `date` is
/// untrusted and only ever parsed by `month_bounds_from_date`; the plan is
/// always the caller's own.
#[procedure]
async fn generate_menu(cx: &Cx, date: String) -> Result<Outcome<bool>> {
    let user = require_user(cx).await?;
    let app = state(cx);

    if let Err(err) = request_generation(app, &user.id, &user.tz, &date).await {
        return Ok(Err(toast_message(cx, &err)));
    }

    let deadline = Instant::now() + CATCH_UP_TIMEOUT;
    loop {
        match generation_caught_up(app, &user.id, &user.tz, &date).await {
            Ok(true) => return Ok(Ok(true)),
            Ok(false) if Instant::now() >= deadline => return Ok(Ok(false)),
            Ok(false) => tokio::time::sleep(CATCH_UP_INTERVAL).await,
            Err(err) => return Ok(Err(toast_message(cx, &err))),
        }
    }
}

#[page("/_tc/menu")]
async fn menu_page(cx: &Cx) -> Result<impl View> {
    let user = require_user(cx).await?;
    let app = state(cx);
    let t = i18n(cx);

    // Tracked server-side read: bumping `rev` in the browser re-runs this page
    // on the server and morphs the result in, keeping focus and scroll.
    let rev = signal(cx, || 0.0);
    let _ = rev.get();
    let pending = signal(cx, || false);
    // A re-run syncs every unfocused input to the server-rendered value, so an
    // input that must survive one is bound to a signal (signals keep their value).
    let note = signal(cx, String::new);

    let bounds = imkitchen_core::mealplan::month_bounds_from_now(&user.tz)?;
    let slots = app
        .core
        .mealplan
        .range(&user.id, bounds.first, bounds.last)
        .await?;

    let fmt = time::macros::format_description!("[year]-[month]-[day]");
    let date = bounds.date.format(&fmt).unwrap_or_default();
    let planned = slots.len();
    let generated = t.t("Meal plan generated");
    let still_projecting = t.t("Your meal plan is being generated, refresh in a moment");

    let days = slots
        .iter()
        .map(|slot| {
            let day = OffsetDateTime::from_unix_timestamp(slot.day as i64)
                .map(|d| d.format(&fmt).unwrap_or_default())
                .unwrap_or_default();
            (day, slot.main_course.name.to_owned())
        })
        .collect::<Vec<_>>();

    Ok(view! {
        base(
            title: t.t("Menu"),
            <main class="container mx-auto px-4 py-16 max-w-2xl">
                <h1 class="text-3xl font-serif font-bold text-ink">(t.t("Menu"))</h1>
                <p class="mt-2 text-ink-2">
                    <span id="planned-count">(planned)</span> " " (t.t("days planned"))
                </p>

                <input
                    id="scratch"
                    class="mt-4 border border-line rounded-xl px-3 py-2"
                    placeholder="signal-bound"
                    :value=$(note.get())
                    @input=$(|e: Event| note.set(e.target.value))
                >
                // Unbound on purpose: documents that a re-run resets it.
                <input id="scratch-unbound" class="mt-4 border border-line rounded-xl px-3 py-2" placeholder="unbound">

                <button
                    id="generate-button"
                    type="button"
                    class="mt-6 bg-primary-500 text-white font-semibold px-6 py-3 rounded-xl hover:bg-primary-600 transition cursor-pointer disabled:opacity-60"
                    :disabled=$(pending.get())
                    @click=$(async |_e| {
                        pending.set(true);
                        let outcome = generate_menu(date).await;
                        pending.set(false);
                        if outcome.is_ok() {
                            if outcome.unwrap() {
                                rev.increment();
                                raw!("imk.toast('success', ${generated})", drop(generated));
                            } else {
                                raw!("imk.toast('success', ${still_projecting})", drop(still_projecting));
                            }
                        } else {
                            let message = outcome.unwrap_err();
                            raw!("imk.toast('error', ${message})", drop(message));
                        }
                    })
                >
                    (t.t("Generate"))
                </button>

                <ul id="planned-days" class="mt-8 space-y-1">
                    for (day, main_course) in days {
                        <li id=(format!("day-{day}")) class="text-ink-2">(day) " — " (main_course)</li>
                    }
                </ul>
            </main>
        )
    })
}

pub fn tc_routes(builder: RouterBuilder) -> RouterBuilder {
    builder.page(menu_page).procedure(generate_menu)
}
