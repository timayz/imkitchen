//! `/groceries`: what who reads the page has to buy to cook their list, to
//! be read in the shop: the ingredients of every meal, for as many as it is
//! cooked for, the same ones added up, by name.
//!
//! The groceries are worked out from the list as app knows it, at every
//! read ([`crate::groceries`]): they are the list's as it stands, and its
//! recipes' as they now are.
//!
//! An item is checked once it is in the basket, and unchecked to buy it
//! again: a command of the api's, so it is checked on every device. Without
//! a script each item is a form that posts to its page; with it
//! (`groceries.js`) the item is checked at once and the command sent behind
//! the page, which is not read again.
//!
//! The script says nothing itself: every sentence it shows is in the page,
//! in the language it is read in.

use reqwest::Method;
use topcoat::{
    Result,
    asset::{Asset, asset},
    context::Cx,
    metadata::Metadata,
    router::{error::see_other, href, page, path_param, query_params},
    view::{View, component, view},
};

use super::{BUTTON, Failure, H1, account, asking, failed, followed, list, read};
use crate::api::CommandError;
use crate::app::{api, db, language, t};

/// What checks and unchecks an item without reading the page again.
pub const SCRIPT: Asset = asset!("./groceries.js");

/// What the api refuses a command on an item its list no longer calls for
/// with.
const ITEM_NOT_FOUND: &str = "grocery_item_not_found";

/// What the address of the groceries says of what just happened: the list
/// changed since the page was read.
const CHANGED: &str = "changed";

path_param!(pub item_id);

#[query_params]
struct Query {
    /// What just happened to the groceries.
    done: Option<String>,
}

/// `quantity` of something counted in `unit`, as a person reads it: "300 g",
/// "2.5". To the thousandth, which the groceries are rounded to.
fn amount(quantity: f64, unit: Option<&str>) -> String {
    let quantity = format!("{quantity:.3}");
    let quantity = quantity.trim_end_matches('0').trim_end_matches('.');
    match unit {
        Some(unit) => format!("{quantity} {unit}"),
        None => quantity.to_owned(),
    }
}

/// Puts the item of the address in the basket (`checked`) or back among
/// what is to buy, then sends the browser back to the groceries once app
/// followed; or says what the api refused.
async fn said(cx: &Cx, checked: bool) -> Result<Failure> {
    let language = language(cx);
    let id = item_id(cx);
    let method = if checked { Method::PUT } else { Method::DELETE };
    let asked = api(cx)
        .command(
            asking(cx, &language),
            method,
            &["groceries", "items", id, "check"],
            None::<&()>,
        )
        .await;
    match asked {
        Ok(()) => {
            let (db, owner_id) = (db(cx), account(cx).id.as_str());
            followed(|| async {
                let groceries = crate::groceries::of(db, owner_id).await?;
                // An item app does not know yet is not waited for.
                Ok(groceries
                    .iter()
                    .find(|grocery| grocery.id == id)
                    .is_none_or(|grocery| grocery.checked == checked))
            })
            .await;
            Err(see_other(href!(show).resolve(cx)).into())
        }
        // The list changed since the page was read, and calls for no such
        // item any more: the groceries are shown as they now stand.
        Err(CommandError::Refused(refusal)) if refusal.code == ITEM_NOT_FOUND => {
            Err(see_other(href!(show).query([("done", CHANGED)]).resolve(cx)).into())
        }
        Err(err) => Failure::of(cx, err),
    }
}

#[page(with(Metadata::new().title(t!(groceries_meta_title))))]
pub async fn show(cx: &Cx) -> Result<impl View> {
    Ok(view! { content(failure: None) })
}

/// Puts an item in the basket.
#[page(POST "./{item_id}/check", with(Metadata::new().title(t!(groceries_meta_title))))]
async fn check(cx: &Cx) -> Result<impl View> {
    let failure = said(cx, true).await?;

    Ok(view! { content(failure: Some(failure)) })
}

/// Takes an item out of the basket: it is to buy again.
#[page(POST "./{item_id}/uncheck", with(Metadata::new().title(t!(groceries_meta_title))))]
async fn uncheck(cx: &Cx) -> Result<impl View> {
    let failure = said(cx, false).await?;

    Ok(view! { content(failure: Some(failure)) })
}

/// An item of the groceries, as it is shown.
struct Row {
    id: String,
    name: String,
    /// How much of it to buy, when a recipe says.
    amount: Option<String>,
    /// How much the list calls for, when the user buys another amount.
    called_for: Option<String>,
    checked: bool,
    /// Where it is checked, and where it is unchecked.
    check: String,
    uncheck: String,
}

/// The groceries, and what `failure` says of the command that was just
/// refused.
#[component]
async fn content(cx: &Cx, failure: Option<Failure>) -> Result<impl View> {
    let groceries = read(crate::groceries::of(db(cx), &account(cx).id).await)?;
    let rows: Vec<Row> = groceries
        .into_iter()
        .map(|grocery| {
            let unit = grocery.unit.as_deref();
            let needed = grocery.quantity.map(|quantity| amount(quantity, unit));
            let bought = grocery
                .adjusted_quantity
                .map(|quantity| amount(quantity, unit));
            Row {
                check: href!(check, item_id: grocery.id.as_str())
                    .resolve(cx)
                    .to_string(),
                uncheck: href!(uncheck, item_id: grocery.id.as_str())
                    .resolve(cx)
                    .to_string(),
                id: grocery.id,
                name: grocery.name,
                called_for: needed.clone().filter(|_| bought.is_some()),
                amount: bought.or(needed),
                checked: grocery.checked,
            }
        })
        .collect();
    let total = rows.len();
    let checked = rows.iter().filter(|row| row.checked).count();
    let changed = failure.is_none()
        && query_params::<Query>(cx)
            .ok()
            .is_some_and(|query| query.done.as_deref() == Some(CHANGED));

    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12" data-groceries="">
            <h1 class=(H1)>(t!(groceries_title))</h1>
            <p class="mt-3 text-sm text-stone-500">(t!(groceries_text))</p>
            if changed {
                <p
                    class="mt-6 rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 text-amber-900"
                    role="status"
                    data-done=(CHANGED)
                >
                    (t!(groceries_changed))
                </p>
            }
            <div class="mt-6">failed(failure: failure)</div>
            // Said by the script, when a command it sent was not done.
            <p
                class="mt-6 rounded-lg border border-red-300 bg-red-50 px-4 py-3 text-red-900"
                role="alert"
                data-groceries-failed=""
                hidden=""
            >
                (t!(command_failed))
            </p>
            if total == 0 {
                <p class="mt-8 text-stone-600" data-empty="">(t!(groceries_none))</p>
                <p class="mt-6">
                    <a class=(BUTTON) href=(href!(list::show))>(t!(groceries_fill))</a>
                </p>
            } else {
                <p class="mt-8 text-sm text-stone-500 tabular-nums" aria-live="polite">
                    <span data-groceries-checked="">(checked)</span>
                    " / "
                    (total)
                    " "
                    (t!(groceries_in_basket))
                </p>
            }
            <ul
                class="mt-4 divide-y divide-stone-200 overflow-hidden rounded-xl border border-stone-200 bg-white empty:hidden"
                data-items=(total)
            >
                for row in rows {
                    <li data-item=(row.id) data-checked=(row.checked.to_string())>
                        <form
                            method="post"
                            action=(if row.checked { row.uncheck.as_str() } else { row.check.as_str() })
                            data-check=(row.check.as_str())
                            data-uncheck=(row.uncheck.as_str())
                        >
                            // The whole row: easy to reach, a basket in
                            // the other hand.
                            <button
                                class="group flex w-full items-center gap-4 px-4 py-4 text-left hover:bg-stone-50"
                                type="submit"
                                role="checkbox"
                                aria-checked=(row.checked.to_string())
                            >
                                <span
                                    class="flex size-6 shrink-0 items-center justify-center rounded-md border border-stone-400 text-sm font-bold text-stone-50 group-aria-checked:border-stone-900 group-aria-checked:bg-stone-900"
                                    aria-hidden="true"
                                >
                                    <span class="invisible group-aria-checked:visible">"✓"</span>
                                </span>
                                <span class="min-w-0 flex-1 text-lg wrap-break-word group-aria-checked:text-stone-400 group-aria-checked:line-through">
                                    (row.name)
                                </span>
                                <span class="shrink-0 text-right group-aria-checked:text-stone-400">
                                    if let Some(amount) = row.amount {
                                        <span class="font-semibold tabular-nums" data-amount=(amount.as_str())>
                                            (amount.as_str())
                                        </span>
                                    }
                                    if let Some(called_for) = row.called_for {
                                        <span
                                            class="block text-xs text-stone-500"
                                            data-called-for=(called_for.as_str())
                                        >
                                            (t!(groceries_called_for, amount: called_for.as_str()))
                                        </span>
                                    }
                                </span>
                            </button>
                        </form>
                    </li>
                }
            </ul>
        </main>
        <script src=(SCRIPT) defer=""></script>
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_amount_is_read_without_the_zeros_it_does_not_need() {
        assert_eq!(amount(300.0, Some("g")), "300 g");
        assert_eq!(amount(2.5, None), "2.5");
        assert_eq!(amount(0.667, Some("tbsp")), "0.667 tbsp");
        assert_eq!(amount(57.0 * 0.001, Some("kg")), "0.057 kg");
        assert_eq!(amount(1000.0, None), "1000");
    }
}
