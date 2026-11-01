use evento::{AggregateExt, WriteBuilder};
use imkitchen_core::recipe::favorite::legacy_id;
use imkitchen_types::favorite::Saved;
use temp_dir::TempDir;

/// A favorite saved before evento alpha.28 lives in a stream keyed by the
/// old `hash_ids`: it must still load as saved, and unsave/save must keep
/// writing to that stream instead of starting a second one.
#[tokio::test]
async fn legacy_stream_is_found_and_kept() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = crate::helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let cmd = imkitchen_core::recipe::Module::new(state.clone());

    let recipe_id = cmd.create("chef", "chef".to_owned()).await?;
    let user_id = "cook";
    let legacy = legacy_id(&recipe_id, user_id);
    assert_ne!(legacy, evento::hash_ids(vec![&recipe_id, user_id]));

    WriteBuilder::new(&legacy)
        .event(&Saved {
            recipe_id: recipe_id.clone(),
            recipe_owner: "chef".to_owned(),
        })
        .requested_by(user_id)
        .commit(&state.executor)
        .await?;

    let favorite = cmd.favorite.load(&recipe_id, user_id).await?;
    assert!(favorite.saved);
    assert_eq!(favorite.id, legacy);

    cmd.favorite.unsave(&recipe_id, user_id).await?;
    let favorite = cmd.favorite.load(&recipe_id, user_id).await?;
    assert!(!favorite.saved);
    assert_eq!(favorite.id, legacy);

    cmd.favorite.save(&recipe_id, "chef", user_id).await?;
    let favorite = cmd.favorite.load(&recipe_id, user_id).await?;
    assert!(favorite.saved);
    assert_eq!(favorite.id, legacy);

    // Three events, all on the legacy stream.
    let version = state.executor.original_version::<Saved>(&legacy).await?;
    assert_eq!(version, Some(3));

    Ok(())
}

/// Without a legacy stream, a favorite is keyed by the current `hash_ids`.
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
