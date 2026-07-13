# Cook's recipes page — click a username to browse that cook

## Context

On the recipe detail page (`/r/{slug}`) the author card shows `@{{ owner_name }}` as
plain text. Users want to click that username and see **only** the recipes shared by
that cook. Since a dedicated cook page will now list a cook's recipes, the small
"More from this cook" right-rail section on the detail page becomes redundant and
should be removed.

**Outcome:** A new **public** page `/cooks/{username}` lists a cook's shared recipes
with the same filter/sort/search/view controls as the community browse (scoped to
that cook). The detail page's author-card username links to it, and the "More from
this cook" section is deleted.

Decisions already confirmed with the user:
- URL: `/cooks/{username}`
- Page scope: **full filters/sort** (recipe-type chips + search + sort + grid/list toggle), scoped to the cook — no Mine / Saved / No-image / Import / New / Share-All controls.

The page is public and mirrors the proven anonymous-visitor pattern in
`detail.rs` (anonymous → `AuthUser::demo()` + `template.demo()`), so guests can
browse and crawlers get real links.

---

## Implementation

### 1. Resolve username → owner_id (query layer)
`crates/core/src/recipe/query/user.rs`

Add a helper on `Module<E>` (next to `find_id_by_slug`, ~line 254) that resolves a
username to its owner id, considering only shared, non-draft recipes:

```rust
pub async fn find_owner_id_by_name(&self, name: impl Into<String>) -> anyhow::Result<Option<String>> {
    // SELECT owner_id FROM recipe_user
    //   WHERE owner_name = ? AND is_shared = 1 AND name <> '' LIMIT 1
}
```
Build it with the same `sea_query` + `build_sqlx` pattern as `find_id_by_slug`.
`owner_name` mirrors `User.username` (unique), so one row is enough. Returns `None`
when the cook has no shared recipes → the handler renders `NotFoundTemplate`.

### 2. Cook page handler (new)
`web/recipe/src/routes/cook.rs` (new) + register `pub mod cook;` in
`web/recipe/src/routes/mod.rs`, and add the route in `web/recipe/src/lib.rs`:
`.route("/cooks/{username}", get(routes::cook::page))`.

Handler shape (mirror `detail::page` for the public/demo handling and reuse
`filter_user`):
- `page(template, user: Option<AuthUser>, Path((username,)): Path<(String,)>, State(app), Query(input): Query<PageQuery>)`.
- Define a local `PageQuery` (like `index::PageQuery` but only: `first/after/last/before`, `recipe_type`, `search`, `sort_by`, `view`).
- Resolve `owner_id = find_owner_id_by_name(&username)`; `None` → `NotFoundTemplate`.
- Anonymous handling copied from `detail.rs:201-212`:
  `let is_anonymous = user.is_none(); let user = user.unwrap_or_else(AuthUser::demo);`
  then `let template = if is_anonymous { template.demo() } else { template };`
- Load header data: `find_user_stat(&owner_id)` (→ `stat.shared`) and
  `identity.user_profile.load(&owner_id)` (→ description), same calls as `detail.rs:214-218,326-329`.
- Query recipes with `filter_user(RecipesQuery { user_id: Some(owner_id), is_shared: Some(true), recipe_type, search, sort_by, args: args.limit(20), .. })` — all other fields `None`/`vec![]`/`false`, `in_meal_plan: None`.
- Render a new `CookTemplate` carrying: `current_path: "recipes"`, `user`,
  `username` (the cook's, for building URLs), `stat`, `owner_description`, `recipes`, `query`.

### 3. Cook page template (new)
`templates/recipes-cook.html` — `{% extends "_user.html" %}`.

Reuse the structure of `templates/recipes-index.html`, trimmed to the public subset:
- **Cook header:** avatar via `{{ username|bg_color }}` + `{{ username|initials }}`,
  `@{{ username }}`, `{{ stat.shared }} {{ "recipes shared"|t }}`, and
  `{{ owner_description|nl2br|safe }}` when non-empty — copy the author-card markup
  from `recipes-detail.html:302-315`.
- **Filter form:** copy the type-chips (`RecipeType::VARIANTS`), search input, sort
  `<select>`, and grid/list view toggle from `recipes-index.html`. **Drop** the Mine,
  Saved (`in_meal_plan`), No-image, Import, New-recipe, and chef Share-All controls.
  Point the live-filter form at the cook URL: `ts-req="{{ "/cooks/"|demo_href }}{{ username }}"`, `ts-target="#recipes-list"`.
- **Results grid/list:** copy the `#recipes-list` block and the grid/list card markup
  (macros `type_emoji`/`type_hero_classes`/`type_pill_classes`/`type_ink_class`/`lazy_thumbnail`)
  from `recipes-index.html:232-335`. Recipe links use `{{ "/r/"|demo_href }}{{ node.slug }}`.
  In the load-more `ts-req` URLs, replace the `/recipes?after=...` base with
  `{{ "/cooks/"|demo_href }}{{ username }}?after=...` and preserve only
  `sort_by`, `recipe_type`, `search`, `view` (no mine/in_meal_plan/no_image).
  Drop the `mine_active`/`is_shared` "Shared" badge (not meaningful here).

### 4. Link the username on the detail page + remove "More from this cook"
`templates/recipes-detail.html`
- Author card (line 308): wrap the username in a link —
  `<a href="{{ "/cooks/"|demo_href }}{{ owner_name }}" class="hover:text-primary-500">@{{ owner_name }}</a>`.
  (Optionally also link the small `@username` in the `suggestion_card` macro at line 94
  and index cards — nice-to-have, not required.)
- Delete the entire "More from this cook" `<section>` (lines 412-423). Keep the
  `suggestion_card` macro — it is still used by "Similar recipes".

`web/recipe/src/routes/detail.rs`
- Remove the `cook_recipes` field from `DetailTemplate` (line 55) and its `Default`
  (line 157) and the render site (line 341).
- Remove the `cook_recipes` `filter_user` call (lines 228-243) and simplify the
  `similar_recipes` exclusion: `exclude_ids` becomes just `vec![recipe.id.to_owned()]`
  (drop the block at lines 245-251 that folded cook_recipes ids in).

### 5. Demo parity (keep the tour link-safe)
`web/demo/src/fixtures.rs`
- In `recipe_detail` (lines 901-906, 924): drop the `cook_nodes`/`cook_recipes` fields
  now that `DetailTemplate` no longer has them.
- Add a `cook(username)` fixture builder returning the new `CookTemplate` populated
  from `catalog()` (as the cook's recipes) with a fixture header (username, shared
  count, description) — mirroring how `recipes()`/`recipe_detail` build fixtures.

`web/demo/src/lib.rs`
- Register `.route("/demo/cooks/{username}", get(cooks))` and add a `cooks` handler:
  `template.demo().render(fixtures::cook(&username))`. This keeps the `demo_href`
  username link inside the demo tour and never 404s.

### 6. Translations
`locales/en.json` and `locales/fr.json` — add any new UI string keys introduced by
`recipes-cook.html` (e.g. a page `{% block title %}`/heading). Reuse existing keys
where possible: `"recipes shared"`, `"All"`, `"Sort"`, `"Recently Added"`,
`"Easiest"`, `"Hardest"`, `"Search by name, description or ingredient..."`,
`"No recipes found"`, `"Grid"`, `"List"` already exist. Add English + French for
anything genuinely new.

---

## Verification

1. **Build/lint:** `cargo build` (and `cargo clippy`) — confirms the `DetailTemplate`
   field removal, new route/module, and templates compile (Askama compiles templates).
2. **Run the app** (`/run` skill or the project's server command) and, signed in:
   - Open a shared recipe's detail page → the author card shows `@username` as a link;
     the right-rail "More from this cook" is gone; "Similar recipes" still renders.
   - Click the username → lands on `/cooks/{username}` showing only that cook's shared
     recipes. Exercise the type chips, search, sort, and grid/list toggle — results
     update live and load-more appends the next page with filters preserved.
   - Open `/cooks/{unknown}` → 404 (NotFound).
3. **Anonymous/guest:** log out (or use a private window) and open a shared recipe →
   click the username → cook page renders in demo mode (links stay under `/demo`,
   actions funnel to sign-up).
4. **Demo tour:** visit `/demo/recipes`, open a recipe, click the author username →
   `/demo/cooks/{username}` renders the fixture cook page (no 404).
5. If tests exist for recipe web routes/queries, run `cargo test` for the recipe
   crates.
