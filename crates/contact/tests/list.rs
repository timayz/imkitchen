use evento::cursor::Args;
use imkitchen_contact::{
    Command, FilterQuery, SortBy, Status, Subject, SubmitContactFormInput, subscribe_list,
    subscribe_stat,
};
use imkitchen_shared::Metadata;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
pub async fn test_contact_query() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_contact::Command(state.evento.clone(), state.pool.clone());
    let query = imkitchen_contact::Query(state.pool.clone());
    let metadata = Metadata::default();

    let unread_contacts = create_submit_contact_form_all(
        &command,
        vec!["unread1", "unread2", "unread3", "unread4", "unread5"],
    )
    .await?;

    let read_contacts =
        create_submit_contact_form_all(&command, vec!["read1", "read2", "read3"]).await?;

    let fut = read_contacts
        .iter()
        .map(|id| command.mark_read_and_reply(id, &metadata));

    futures::future::join_all(fut).await;

    let resolved_contacts =
        create_submit_contact_form_all(&command, vec!["resolved1", "resolved2"]).await?;

    let fut = resolved_contacts
        .iter()
        .map(|id| command.resolve(id, &metadata));

    futures::future::join_all(fut).await;

    subscribe_list()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    subscribe_stat()
        .data(state.pool.clone())
        .data(query.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    let stat = query.find_stat(0).await?.unwrap();
    assert_eq!(
        stat.total,
        (unread_contacts.len() + read_contacts.len() + resolved_contacts.len()) as u32
    );

    let contacts = query
        .filter(FilterQuery {
            status: Some(Status::Resolved),
            subject: None,
            sort_by: SortBy::MostRecent,
            args: Args::forward(20, None),
        })
        .await?;

    for edge in contacts.edges {
        assert!(edge.node.email.starts_with("resolved"));
    }

    let contacts = query
        .filter(FilterQuery {
            status: Some(Status::Unread),
            subject: None,
            sort_by: SortBy::MostRecent,
            args: Args::forward(20, None),
        })
        .await?;

    for edge in contacts.edges {
        assert!(edge.node.email.starts_with("unread"));
    }

    let contacts = query
        .filter(FilterQuery {
            status: Some(Status::Read),
            subject: None,
            sort_by: SortBy::MostRecent,
            args: Args::forward(20, None),
        })
        .await?;

    for edge in contacts.edges {
        assert!(edge.node.email.starts_with("read"));
    }

    Ok(())
}

pub async fn create_submit_contact_form_all(
    command: &Command<evento::Sqlite>,
    names: impl IntoIterator<Item = impl Into<String>>,
) -> anyhow::Result<Vec<String>> {
    let mut ids = vec![];
    for name in names.into_iter() {
        let name = name.into();
        let id = command
            .submit_contact_form(
                SubmitContactFormInput {
                    email: format!("{name}@imkitchen.localhost"),
                    name: "my name".to_owned(),
                    subject: Subject::Other,
                    message: "my message".to_owned(),
                },
                &Metadata::default(),
            )
            .await?;
        ids.push(id);
    }

    Ok(ids)
}
