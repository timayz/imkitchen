use temp_dir::TempDir;

/// A follow is keyed by `hash_ids([chef_id, user_id])`; following twice is a
/// no-op and unfollowing clears it.
#[tokio::test]
async fn follow_and_unfollow_a_chef() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = crate::helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let cmd = imkitchen_core::recipe::Module::new(state.clone());

    let follow = cmd.follow.load("chef", "cook").await?;
    assert!(!follow.following);

    cmd.follow.follow("chef", "pastrychef", "cook").await?;
    cmd.follow.follow("chef", "pastrychef", "cook").await?;
    let follow = cmd.follow.load("chef", "cook").await?;
    assert!(follow.following);
    assert_eq!(follow.id, evento::hash_ids(vec!["chef", "cook"]));
    assert_eq!(
        follow.aggregate_version, 1,
        "a second follow writes nothing"
    );

    cmd.follow.unfollow("chef", "cook").await?;
    let follow = cmd.follow.load("chef", "cook").await?;
    assert!(!follow.following);

    Ok(())
}

/// The read model lists the chef for the follower and counts the follower on
/// the chef's stat row; replaying the same events changes neither.
#[tokio::test]
async fn follower_read_model_lists_and_counts() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = crate::helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let cmd = imkitchen_core::recipe::Module::new(state.clone());

    cmd.follow.follow("chef", "pastrychef", "cook").await?;
    cmd.follow.follow("chef", "pastrychef", "other").await?;
    run_follower_subscription(&state).await?;

    let chefs = cmd.list_followed_chefs("cook").await?;
    assert_eq!(chefs.len(), 1);
    assert_eq!(chefs[0].chef_id, "chef");
    assert_eq!(chefs[0].chef_name, "pastrychef");
    assert_eq!(chefs[0].follower, 2);
    assert_eq!(chefs[0].shared, 0);
    let stat = cmd.find_user_stat("chef").await?.unwrap_or_default();
    assert_eq!(stat.follower, 2);

    // A fresh subscription cursor replays everything: still one row, still 2.
    reset_follower_cursor(&state).await?;
    run_follower_subscription(&state).await?;
    assert_eq!(cmd.list_followed_chefs("cook").await?.len(), 1);
    assert_eq!(
        cmd.find_user_stat("chef")
            .await?
            .unwrap_or_default()
            .follower,
        2
    );

    cmd.follow.unfollow("chef", "cook").await?;
    run_follower_subscription(&state).await?;
    assert!(cmd.list_followed_chefs("cook").await?.is_empty());
    assert_eq!(cmd.list_followed_chefs("other").await?.len(), 1);
    assert_eq!(
        cmd.find_user_stat("chef")
            .await?
            .unwrap_or_default()
            .follower,
        1
    );

    Ok(())
}

async fn run_follower_subscription(
    state: &imkitchen_core::State<evento::Sqlite>,
) -> anyhow::Result<()> {
    imkitchen_core::recipe::query::follower::subscription()
        .data(state.write_db.clone())
        .any_routing_key()
        .no_retry()
        .run_once(&state.executor)
        .await?;
    Ok(())
}

async fn reset_follower_cursor(
    state: &imkitchen_core::State<evento::Sqlite>,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE subscriber SET cursor = NULL WHERE key = 'recipe-follower-view'")
        .execute(&state.write_db)
        .await?;
    Ok(())
}
