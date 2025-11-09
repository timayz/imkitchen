use evento::cursor::Args;
use imkitchen::{AdminUserInput, AdminUserStatus};
use imkitchen_shared::Metadata;
use imkitchen_user::{MadeAdminInput, SuspendInput, ToggleLifePremiumInput};

mod helpers;

#[tokio::test]
pub async fn test_admin_user_query() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());

    let free_tier_users = helpers::create_users(
        &state,
        vec![
            "free.tier1",
            "free.tier2",
            "free.tier3",
            "free.tier4",
            "free.tier5",
        ],
    )
    .await?
    .into_iter()
    .rev()
    .collect::<Vec<_>>();

    let premium_users = helpers::create_users(&state, vec!["premium1", "premium2", "premium3"])
        .await?
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    let fut = premium_users.iter().map(|id| {
        command.toggle_life_premium(
            ToggleLifePremiumInput { id: id.to_owned() },
            Metadata::default(),
        )
    });

    futures::future::join_all(fut).await;

    let admin_users = helpers::create_users(&state, vec!["admin1", "admin2"])
        .await?
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    let fut = admin_users
        .iter()
        .map(|id| command.made_admin(MadeAdminInput { id: id.to_owned() }, Metadata::default()));

    futures::future::join_all(fut).await;

    let suspend_users =
        helpers::create_users(&state, vec!["suspend1", "suspend2", "suspend3", "suspend4"])
            .await?
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

    let fut = suspend_users
        .iter()
        .map(|id| command.suspend(SuspendInput { id: id.to_owned() }, Metadata::default()));

    futures::future::join_all(fut).await;

    imkitchen::subscribe_admin_user()
        .data(state.pool.clone())
        .oneshot(&state.evento)
        .await?;

    imkitchen::subscribe_global_stat()
        .data(state.pool.clone())
        .oneshot(&state.evento)
        .await?;

    let stats = imkitchen::query_admin_users_global_stats(&state.pool).await?;
    assert_eq!(
        stats.total,
        (free_tier_users.len() + premium_users.len() + admin_users.len() + suspend_users.len())
            as u32
    );

    let users = imkitchen::query_admin_users(
        &state.pool,
        AdminUserInput {
            status: Some(AdminUserStatus::Active),
            account_type: None,
            sort_by: imkitchen::AdminUserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        },
    )
    .await?;

    let mut expected_users = admin_users.to_vec();
    expected_users.extend(premium_users.iter().cloned());
    expected_users.extend(free_tier_users.iter().cloned());

    for (pos, user) in expected_users.iter().enumerate() {
        assert_eq!(users.edges[pos].node.id.as_str(), user);
    }

    let users = imkitchen::query_admin_users(
        &state.pool,
        AdminUserInput {
            status: Some(AdminUserStatus::Suspended),
            account_type: None,
            sort_by: imkitchen::AdminUserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        },
    )
    .await?;

    for (pos, user) in suspend_users.iter().enumerate() {
        assert_eq!(users.edges[pos].node.id.as_str(), user);
    }

    let users = imkitchen::query_admin_users(
        &state.pool,
        AdminUserInput {
            status: None,
            account_type: Some(imkitchen::AdminUserAccountType::Admin),
            sort_by: imkitchen::AdminUserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        },
    )
    .await?;

    for (pos, user) in admin_users.iter().enumerate() {
        assert_eq!(users.edges[pos].node.id.as_str(), user);
    }

    let users = imkitchen::query_admin_users(
        &state.pool,
        AdminUserInput {
            status: None,
            account_type: Some(imkitchen::AdminUserAccountType::FreeTier),
            sort_by: imkitchen::AdminUserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        },
    )
    .await?;

    let mut expected = suspend_users.to_vec();
    expected.extend(free_tier_users.iter().cloned());

    for (pos, user) in expected.iter().enumerate() {
        assert_eq!(users.edges[pos].node.id.as_str(), user);
    }

    let users = imkitchen::query_admin_users(
        &state.pool,
        AdminUserInput {
            status: None,
            account_type: Some(imkitchen::AdminUserAccountType::Premium),
            sort_by: imkitchen::AdminUserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        },
    )
    .await?;

    for (pos, user) in premium_users.iter().enumerate() {
        assert_eq!(users.edges[pos].node.id.as_str(), user);
    }

    Ok(())
}
