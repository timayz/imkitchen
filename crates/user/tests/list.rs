use evento::cursor::Args;
use imkitchen_shared::Metadata;
use imkitchen_user::{FilterQuery, Role, State, UserSortBy};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
pub async fn test_admin_user_query() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_user::Command {
        evento: state.evento.clone(),
        read_db: state.pool.clone(),
        write_db: state.pool.clone(),
    };
    let query = imkitchen_user::Query(state.pool.clone());
    let subscription_command =
        imkitchen_user::subscription::Command(state.evento.clone(), state.pool.clone());
    let metadata = Metadata::default();

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
    .await?;

    let premium_users =
        helpers::create_users(&state, vec!["premium1", "premium2", "premium3"]).await?;

    let fut = premium_users
        .iter()
        .map(|id| subscription_command.toggle_life_premium(id, &metadata));

    futures::future::join_all(fut).await;

    let admin_users = helpers::create_users(&state, vec!["admin1", "admin2"]).await?;

    let fut = admin_users
        .iter()
        .map(|id| command.made_admin(id, &metadata));

    futures::future::join_all(fut).await;

    let suspend_users =
        helpers::create_users(&state, vec!["suspend1", "suspend2", "suspend3", "suspend4"]).await?;

    let fut = suspend_users
        .iter()
        .map(|id| command.suspend(id, &metadata));

    futures::future::join_all(fut).await;

    imkitchen_user::subscribe_list()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    imkitchen_user::subscribe_stat()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    let stat = query.find_stat(0).await?.unwrap();
    assert_eq!(
        stat.total,
        (free_tier_users.len() + premium_users.len() + admin_users.len() + suspend_users.len())
            as u32
    );

    let users = query
        .filter(FilterQuery {
            state: Some(State::Active),
            role: None,
            sort_by: UserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        })
        .await?;

    for edge in users.edges {
        assert!(
            edge.node.email.starts_with("admin")
                || edge.node.email.starts_with("premium")
                || edge.node.email.starts_with("free.tier")
        );
    }

    let users = query
        .filter(FilterQuery {
            state: Some(State::Suspended),
            role: None,
            sort_by: UserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        })
        .await?;

    for edge in users.edges {
        assert!(edge.node.email.starts_with("suspend"));
    }

    let users = query
        .filter(FilterQuery {
            state: None,
            role: Some(Role::Admin),
            sort_by: UserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        })
        .await?;

    for edge in users.edges {
        assert!(edge.node.email.starts_with("admin"));
    }

    let users = query
        .filter(FilterQuery {
            state: None,
            role: Some(Role::User),
            sort_by: UserSortBy::RecentlyJoined,
            args: Args::forward(20, None),
        })
        .await?;

    for edge in users.edges {
        assert!(!edge.node.is_admin());
    }

    Ok(())
}
