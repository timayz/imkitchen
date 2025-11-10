use evento::cursor::Args;
use imkitchen::{AdminUserInput, AdminUserStatus};
use imkitchen_shared::Metadata;

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

    let fut = premium_users
        .iter()
        .map(|id| command.toggle_life_premium(id, Metadata::default()));

    futures::future::join_all(fut).await;

    let admin_users = helpers::create_users(&state, vec!["admin1", "admin2"])
        .await?
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    let fut = admin_users
        .iter()
        .map(|id| command.made_admin(id, Metadata::default()));

    futures::future::join_all(fut).await;

    let suspend_users =
        helpers::create_users(&state, vec!["suspend1", "suspend2", "suspend3", "suspend4"])
            .await?
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

    let fut = suspend_users
        .iter()
        .map(|id| command.suspend(id, Metadata::default()));

    futures::future::join_all(fut).await;

    imkitchen_user::subscribe_command()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
        .await?;

    imkitchen::subscribe_admin_user()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
        .await?;

    imkitchen::subscribe_global_stat()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
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

    for edge in users.edges {
        assert!(
            edge.node.email.starts_with("admin")
                || edge.node.email.starts_with("premium")
                || edge.node.email.starts_with("free.tier")
        );
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

    for edge in users.edges {
        assert!(edge.node.email.starts_with("suspend"));
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

    for edge in users.edges {
        assert!(edge.node.email.starts_with("admin"));
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

    for edge in users.edges {
        assert!(edge.node.email.starts_with("suspend") || edge.node.email.starts_with("free.tier"));
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

    for edge in users.edges {
        assert!(edge.node.email.starts_with("premium"));
    }

    Ok(())
}
