use evento::cursor::Args;
use imkitchen::{ContactInput, ContactSortBy};
use imkitchen_contact::ContactStatus;
use imkitchen_shared::Metadata;

mod helpers;

#[tokio::test]
pub async fn test_contact_query() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_contact::Command(state.evento.clone(), state.pool.clone());

    let unread_contacts = helpers::create_submit_contact_form_all(
        &state,
        vec!["unread1", "unread2", "unread3", "unread4", "unread5"],
    )
    .await?
    .into_iter()
    .rev()
    .collect::<Vec<_>>();

    let read_contacts =
        helpers::create_submit_contact_form_all(&state, vec!["read1", "read2", "read3"])
            .await?
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

    let fut = read_contacts
        .iter()
        .map(|id| command.mark_as_read_and_replay(id, Metadata::default()));

    futures::future::join_all(fut).await;

    let resolved_contacts =
        helpers::create_submit_contact_form_all(&state, vec!["resolved1", "resolved2"])
            .await?
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

    let fut = resolved_contacts
        .iter()
        .map(|id| command.resolve(id, Metadata::default()));

    futures::future::join_all(fut).await;

    imkitchen::subscribe_contact()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
        .await?;

    imkitchen::subscribe_global_stat()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
        .await?;

    let stats = imkitchen::query_contact_global_stats(&state.pool).await?;
    assert_eq!(
        stats.total,
        (unread_contacts.len() + read_contacts.len() + resolved_contacts.len()) as u32
    );

    let contacts = imkitchen::query_contacts(
        &state.pool,
        ContactInput {
            status: Some(ContactStatus::Resolved),
            subject: None,
            sort_by: ContactSortBy::MostRecent,
            args: Args::forward(20, None),
        },
    )
    .await?;

    for edge in contacts.edges {
        assert!(edge.node.email.starts_with("resolved"));
    }

    let contacts = imkitchen::query_contacts(
        &state.pool,
        ContactInput {
            status: Some(ContactStatus::Unread),
            subject: None,
            sort_by: ContactSortBy::MostRecent,
            args: Args::forward(20, None),
        },
    )
    .await?;

    for edge in contacts.edges {
        assert!(edge.node.email.starts_with("unread"));
    }

    let contacts = imkitchen::query_contacts(
        &state.pool,
        ContactInput {
            status: Some(ContactStatus::Read),
            subject: None,
            sort_by: ContactSortBy::MostRecent,
            args: Args::forward(20, None),
        },
    )
    .await?;

    for edge in contacts.edges {
        assert!(edge.node.email.starts_with("read"));
    }

    Ok(())
}
