use imkitchen_shared::Metadata;

mod helpers;

#[tokio::test]
async fn test_delete() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;
    let loaded = command.load(&recipe).await?;
    assert!(!loaded.item.deleted);

    command
        .delete_with(loaded, &Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;

    assert!(loaded.item.deleted);

    let err = command
        .delete_with(loaded, &Metadata::by(john.to_owned()))
        .await
        .unwrap_err();

    assert_eq!(err.to_string(), "recipe already deleted".to_owned());

    Ok(())
}
