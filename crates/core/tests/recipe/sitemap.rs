use imkitchen_core::recipe::Module;
use temp_dir::TempDir;

/// Seeds a `recipe_user` row with the fields the sitemap queries filter on.
/// No FTS row needed — the sitemap queries never touch `recipe_user_fts`.
async fn seed(
    db: &sqlx::SqlitePool,
    id: &str,
    name: &str,
    owner_name: Option<&str>,
    is_shared: bool,
    created_at: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO recipe_user \
         (id, cursor, owner_id, owner_name, recipe_type, slug, name, description, ingredients, \
          instructions, dietary_restrictions, is_shared, created_at, difficulty_score) \
         VALUES (?, ?, 'owner-1', ?, 'MainCourse', ?, ?, '', X'', X'', '[]', ?, ?, 1)",
    )
    .bind(id)
    .bind(id) // cursor
    .bind(owner_name)
    .bind(id) // slug — unique per row
    .bind(name)
    .bind(is_shared)
    .bind(created_at)
    .execute(db)
    .await?;

    Ok(())
}

/// `list_shared_slugs` returns only shared, non-draft recipes, newest first;
/// `list_shared_cook_names` returns each cook once, skipping NULL owner names.
#[tokio::test]
async fn test_sitemap_lists_shared_recipes_and_cooks() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = crate::helpers::setup_test_state(path).await?;
    let db = state.read_db.clone();
    let cmd = Module::new(state);

    seed(&db, "shared-old", "tarte", Some("alice"), true, 100).await?;
    seed(&db, "shared-new", "gratin", Some("alice"), true, 200).await?;
    seed(&db, "shared-bob", "ragout", Some("bob"), true, 150).await?;
    seed(&db, "private", "secret pie", Some("carol"), false, 300).await?;
    seed(&db, "draft", "", Some("alice"), true, 400).await?;
    seed(&db, "no-owner-name", "soupe", None, true, 250).await?;

    let slugs = cmd.list_shared_slugs().await?;
    assert_eq!(
        slugs,
        vec!["no-owner-name", "shared-new", "shared-bob", "shared-old"],
        "only shared non-draft recipes, newest first"
    );

    let cooks = cmd.list_shared_cook_names().await?;
    assert_eq!(
        cooks,
        vec!["alice", "bob"],
        "distinct cooks with shared recipes, NULL owner_name and private/draft owners excluded"
    );

    Ok(())
}
