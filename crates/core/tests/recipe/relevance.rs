use evento::cursor::{Args, Value};
use imkitchen_core::recipe::Module;
use imkitchen_core::recipe::query::user::{RecipesQuery, SortBy};
use temp_dir::TempDir;

const RECIPE_A: &str = "recipe_choco_aaaaaaaaaaaaaa";
const RECIPE_B: &str = "recipe_choco_bbbbbbbbbbbbbb";
const RECIPE_C: &str = "recipe_choco_cccccccccccccc";

/// Seeds a row into both the `recipe_user` read model and the `recipe_user_fts`
/// virtual table — mirroring what the projection + FTS subscription write at
/// runtime — so `filter_user` can be exercised directly. The `ingredients` /
/// `instructions` blobs are irrelevant to these assertions (they are not part
/// of `UserViewList`), so we store empty blobs.
async fn seed(
    db: &sqlx::SqlitePool,
    id: &str,
    name: &str,
    description: &str,
    difficulty: i64,
    created_at: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO recipe_user \
         (id, cursor, owner_id, recipe_type, slug, name, description, ingredients, \
          instructions, dietary_restrictions, is_shared, created_at, difficulty_score) \
         VALUES (?, ?, 'owner-1', 'MainCourse', ?, ?, ?, X'', X'', '[]', 1, ?, ?)",
    )
    .bind(id)
    .bind(id) // cursor
    .bind(id) // slug — unique per row
    .bind(name)
    .bind(description)
    .bind(created_at)
    .bind(difficulty)
    .execute(db)
    .await?;

    sqlx::query(
        "INSERT INTO recipe_user_fts (id, name, description, ingredients) VALUES (?, ?, ?, '')",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .execute(db)
    .await?;

    Ok(())
}

fn query(sort_by: SortBy, search: Option<&str>, first: u16, after: Option<Value>) -> RecipesQuery {
    RecipesQuery {
        exclude_ids: None,
        user_id: None,
        recipe_type: None,
        is_shared: None,
        has_thumbnail: None,
        dietary_restrictions: vec![],
        dietary_where_any: false,
        in_meal_plan: None,
        sort_by,
        search: search.map(str::to_owned),
        args: Args {
            first: Some(first),
            after,
            last: None,
            before: None,
        },
    }
}

/// Seeds three recipes whose relevance for "chocolate" is strictly A > B > C
/// (by term frequency in the name), while difficulty is the exact opposite
/// (A easiest, C hardest). So `Hardest` would yield C, B, A — proving any
/// relevance-ordered result is the override, not the requested sort.
async fn seed_chocolate(db: &sqlx::SqlitePool) -> anyhow::Result<()> {
    seed(
        db,
        RECIPE_A,
        "chocolate chocolate chocolate chocolate",
        "rich",
        1,
        100,
    )
    .await?;
    seed(db, RECIPE_B, "chocolate chocolate cake", "sweet", 2, 200).await?;
    seed(db, RECIPE_C, "chocolate tart", "a dessert", 3, 300).await?;
    Ok(())
}

/// A non-empty search term orders results by FTS relevance ("best match first")
/// and overrides whatever `sort_by` was requested.
#[tokio::test]
async fn test_search_orders_by_relevance_overriding_sort_by() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = crate::helpers::setup_test_state(path).await?;
    let db = state.read_db.clone();
    let cmd = Module::new(state);

    seed_chocolate(&db).await?;

    let result = cmd
        .filter_user(query(SortBy::Hardest, Some("chocolate"), 20, None))
        .await?;

    let ids: Vec<&str> = result.edges.iter().map(|e| e.node.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![RECIPE_A, RECIPE_B, RECIPE_C],
        "search must order by relevance (A>B>C), overriding sort_by=Hardest (which would be C,B,A)"
    );

    Ok(())
}

/// Keyset pagination works while searching: paging one recipe at a time with the
/// previous page's `end_cursor` walks the full relevance order with no gaps or
/// duplicates, and `has_next_page` flips off on the last page.
#[tokio::test]
async fn test_relevance_keyset_pagination() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = crate::helpers::setup_test_state(path).await?;
    let db = state.read_db.clone();
    let cmd = Module::new(state);

    seed_chocolate(&db).await?;

    let expected = [RECIPE_A, RECIPE_B, RECIPE_C];
    let mut after: Option<Value> = None;
    let mut seen = vec![];

    for (i, want) in expected.iter().enumerate() {
        let result = cmd
            // sort_by=Hardest throughout to prove the override holds on every page.
            .filter_user(query(SortBy::Hardest, Some("chocolate"), 1, after.clone()))
            .await?;

        assert_eq!(
            result.edges.len(),
            1,
            "page {i} should hold exactly one recipe"
        );
        let node_id = result.edges[0].node.id.clone();
        assert_eq!(&node_id.as_str(), want, "page {i} out of relevance order");
        seen.push(node_id);

        let last_page = i == expected.len() - 1;
        assert_eq!(
            result.page_info.has_next_page, !last_page,
            "has_next_page wrong on page {i}"
        );

        after = result.page_info.end_cursor.clone();
    }

    assert_eq!(
        seen,
        expected.to_vec(),
        "keyset walk must cover the full relevance order exactly once"
    );

    Ok(())
}

/// With no search term the previous behavior is preserved: `sort_by` governs.
/// `RecentlyAdded` orders by `created_at` descending, so the newest comes first.
#[tokio::test]
async fn test_empty_search_keeps_sort_by() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = crate::helpers::setup_test_state(path).await?;
    let db = state.read_db.clone();
    let cmd = Module::new(state);

    seed_chocolate(&db).await?;

    let result = cmd
        .filter_user(query(SortBy::RecentlyAdded, None, 20, None))
        .await?;

    let ids: Vec<&str> = result.edges.iter().map(|e| e.node.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![RECIPE_C, RECIPE_B, RECIPE_A],
        "no search: RecentlyAdded orders by created_at desc (newest C first)"
    );

    Ok(())
}
