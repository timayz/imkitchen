//! SPIKE: topcoat recipes slice, served at `/_tc/recipes`.
//!
//! Proves live search and infinite scroll without twinspark: the list is a
//! `#[shard]` re-rendered on the server when its arguments change. Coalescing
//! and aborting the in-flight request are native, which retires
//! `ts-req-strategy="last"` and the two inline scripts that fought twinspark's
//! event model. The scroll sentinel is driven by `tc-bridge.js`'s `tc-visible`
//! event, as the runtime has no visibility trigger.

use evento::cursor::Args;
use imkitchen_core::recipe::query::user::{RecipesQuery, SortBy};
use imkitchen_web_shared::tc::{
    auth::require_user,
    context::{i18n, state},
    view::base,
};
use topcoat::{
    Result,
    context::Cx,
    router::{RouterBuilder, page},
    runtime::{Event, RouterBuilderShardExt, Signal, shard, signal},
    view::{View, view},
};

const PAGE_SIZE: u16 = 20;
/// `pages` comes from the browser: clamp it so a forged value cannot ask for
/// an unbounded read.
const MAX_PAGES: f64 = 25.0;

/// The caller's own recipes matching `search`, `pages` pages deep.
///
/// Self-authorizing: page guards do not run for shard endpoints. Both
/// arguments are untrusted; the query is always scoped to the caller's id.
///
/// `pages` is owned by the page (so typing a new search can reset it) and read
/// tracked here, so bumping it re-renders only this shard. Items carry an `id`,
/// so the morph keeps loaded rows in place and appends the new ones.
#[shard]
async fn recipes_list(cx: &Cx, search: String, pages: Signal<f64>) -> Result<impl View> {
    let user = require_user(cx).await?;
    let t = i18n(cx);

    let wanted = pages.get().clamp(1.0, MAX_PAGES) as u16;
    let search = Some(search.trim().to_owned()).filter(|s| !s.is_empty());

    let recipes = state(cx)
        .core
        .recipe
        .filter_user(RecipesQuery {
            exclude_ids: None,
            user_id: Some(user.id.to_owned()),
            recipe_type: None,
            is_shared: None,
            has_thumbnail: None,
            dietary_restrictions: vec![],
            dietary_where_any: false,
            in_meal_plan: None,
            sort_by: SortBy::default(),
            search,
            args: Args::forward(PAGE_SIZE * wanted, None),
        })
        .await?;

    let has_next_page = recipes.page_info.has_next_page;
    let count = recipes.edges.len();

    Ok(view! {
        <p class="mt-4 text-sm text-ink-2"><span id="recipes-count">(count)</span> " " (t.t("recipes"))</p>

        <ul id="recipes-list" class="mt-4 space-y-2">
            for edge in recipes.edges {
                <li id=(format!("recipe-{}", edge.node.id)) class="bg-paper border border-line-2 rounded-xl px-4 py-3 h-16">
                    <a href=(format!("/r/{}", edge.node.slug)) class="font-semibold text-ink">(edge.node.name)</a>
                </li>
            }
        </ul>

        if has_next_page {
            <div id="recipes-more" data-tc-visible="" @tc-visible=$(|_e| pages.increment()) class="h-10"></div>
        }
    })
}

#[page("/_tc/recipes")]
async fn recipes_page(cx: &Cx) -> Result<impl View> {
    require_user(cx).await?;
    let t = i18n(cx);

    let search = signal(cx, String::new);
    let pages = signal(cx, || 1.0);

    Ok(view! {
        base(
            title: t.t("Recipes"),
            <main class="container mx-auto px-4 py-16 max-w-2xl">
                <h1 class="text-3xl font-serif font-bold text-ink">(t.t("Recipes"))</h1>

                <input
                    id="recipes-search"
                    type="search"
                    placeholder=(t.t("Search"))
                    class="mt-6 w-full px-4 py-3 border border-line rounded-xl"
                    :value=$(search.get())
                    @input=$(|e: Event| {
                        pages.set(1.0);
                        search.set(e.target.value);
                    })
                >

                recipes_list(search: $(search.get()), pages: $(pages))
            </main>
        )
    })
}

pub fn tc_routes(builder: RouterBuilder) -> RouterBuilder {
    builder.page(recipes_page).shard(recipes_list)
}
