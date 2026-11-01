use temp_dir::TempDir;

/// A favorite is keyed by `hash_ids([recipe_id, user_id])`.
#[tokio::test]
async fn new_favorites_use_hash_ids() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = crate::helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let cmd = imkitchen_core::recipe::Module::new(state.clone());

    let recipe_id = cmd.create("chef", "chef".to_owned()).await?;
    let favorite = cmd.favorite.load(&recipe_id, "cook").await?;
    assert!(!favorite.saved);

    cmd.favorite.save(&recipe_id, "chef", "cook").await?;
    let favorite = cmd.favorite.load(&recipe_id, "cook").await?;
    assert!(favorite.saved);
    assert_eq!(favorite.id, evento::hash_ids(vec![&recipe_id, "cook"]));

    Ok(())
}
