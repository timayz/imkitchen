use crate::helpers;
use imkitchen_core::shopping::{SetCheckedInput, ToggleInput};
use temp_dir::TempDir;

/// `set_checked` is absolute: repeating it is a no-op (no extra event), so an
/// offline client can replay it safely, while `toggle` keeps flipping.
#[tokio::test]
async fn test_set_checked_is_idempotent() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let recipe_id = helpers::import_recipe(&recipe_cmd, "Bread", "flour", 500, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;
    shopping.add_recipe(&recipe_id, 4, "john").await?;
    let flour = shopping.state("john", 4).await?.ingredients[0].key();

    let check = |checked: bool| {
        shopping.set_checked(
            SetCheckedInput {
                name: flour.clone(),
                checked,
            },
            "john",
        )
    };

    check(true).await?;
    let version = shopping.load("john").await?.unwrap().aggregate_version;
    check(true).await?;
    let view = shopping.state("john", 4).await?;
    assert!(view.checked.contains(&flour));
    assert_eq!(
        shopping.load("john").await?.unwrap().aggregate_version,
        version,
        "checking an already checked ingredient writes nothing"
    );

    check(false).await?;
    assert!(!shopping.state("john", 4).await?.checked.contains(&flour));
    check(false).await?;
    assert!(!shopping.state("john", 4).await?.checked.contains(&flour));

    // The toggle still flips.
    shopping
        .toggle(
            ToggleInput {
                name: flour.clone(),
            },
            "john",
        )
        .await?;
    assert!(shopping.state("john", 4).await?.checked.contains(&flour));

    assert!(
        shopping
            .set_checked(
                SetCheckedInput {
                    name: "nope-".to_owned(),
                    checked: true
                },
                "john"
            )
            .await
            .is_err(),
        "an ingredient not on the list cannot be checked"
    );

    Ok(())
}
