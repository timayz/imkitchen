//! Writing to imkitchen, through the api's routes: by anyone, signed in or
//! not, each message kept as it was written.

mod helpers;

use axum::http::header::{ACCEPT_LANGUAGE, CONTENT_TYPE};
use axum::http::{Method, Request, StatusCode};
use helpers::api::{Api, invalid, start};
use imkitchen_aggregates::contact::{ContactMessage, Sent};
use serde_json::{Value, json};
use tower::ServiceExt;

const MESSAGES: &str = "/v1/contact-messages";

/// The one event of the message `id`.
async fn sent(api: &Api, id: &str) -> Sent {
    let stream = evento::read::<ContactMessage>(id)
        .execute(&api.state.executor)
        .await
        .unwrap();
    assert_eq!(stream.len(), 1);
    let event = stream.first().unwrap();
    evento::metadata::Event::<Sent>::try_from(event)
        .unwrap()
        .data
}

#[tokio::test]
async fn anyone_writes_and_the_message_is_kept_as_written() {
    let (api, _outbox, node, _dir) = start().await;

    // Nobody is signed in.
    let body = json!({
        "name": "  Alice  ",
        "email": " Alice@Example.com ",
        "message": "\nHello!\nA second line.  ",
    });
    let (status, answer) = api.post(MESSAGES, None, body.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{answer}");
    let id = answer["id"].as_str().unwrap();
    let message = sent(&api, id).await;
    assert_eq!(message.name, "Alice");
    assert_eq!(message.email, "alice@example.com");
    assert_eq!(message.message, "Hello!\nA second line.");
    assert_eq!(message.language, "en");

    // The same again is another message, in the language it asks for.
    let request = Request::builder()
        .method(Method::POST)
        .uri(MESSAGES)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT_LANGUAGE, "fr-FR,fr;q=0.9")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap();
    let response = api.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let answer: Value = serde_json::from_slice(&bytes).unwrap();
    let other = answer["id"].as_str().unwrap();
    assert_ne!(other, id);
    assert_eq!(sent(&api, other).await.language, "fr");

    api.shutdown().await;
    node.cluster.shutdown().await;
}

#[tokio::test]
async fn a_message_that_says_nothing_is_refused_field_by_field() {
    let (api, _outbox, node, _dir) = start().await;

    let body = json!({ "name": "  ", "email": "nobody", "message": "x".repeat(4001) });
    let (status, answer) = api.post(MESSAGES, None, body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        answer,
        invalid(json!([
            {
                "field": "email",
                "code": "invalid_email",
                "message": "This is not a valid e-mail address.",
            },
            {
                "field": "message",
                "code": "length",
                "message": "This must be between 1 and 4000 characters.",
                "params": { "min": 1, "max": 4000 },
            },
            {
                "field": "name",
                "code": "length",
                "message": "This must be between 1 and 100 characters.",
                "params": { "min": 1, "max": 100 },
            },
        ]))
    );

    api.shutdown().await;
    node.cluster.shutdown().await;
}
