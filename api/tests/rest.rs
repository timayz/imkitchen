//! What every endpoint follows (`api/README.md`): the version in the
//! address, the one shape of the refusals, their language, and the
//! reference generated from the handlers.

mod helpers;

use axum::Router;
use axum::body::Body;
use axum::http::header::{ACCEPT_LANGUAGE, CONTENT_LANGUAGE, CONTENT_TYPE};
use axum::http::{HeaderMap, Method, Request, StatusCode};
use helpers::TestNode;
use http_body_util::BodyExt;
use imkitchen_api::server;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

/// The api's routes over a node alone.
async fn start() -> (Router, TestNode, TempDir) {
    let (node, dir) = TestNode::alone().await;
    (server::router(node.state()), node, dir)
}

/// One request, in `language` when given; the body as it came.
async fn raw(
    router: &Router,
    method: Method,
    path: &str,
    language: Option<&str>,
    body: Option<&str>,
) -> (StatusCode, HeaderMap, String) {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(language) = language {
        request = request.header(ACCEPT_LANGUAGE, language);
    }
    let request = match body {
        Some(body) => request
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_owned())),
        None => request.body(Body::empty()),
    }
    .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = body.collect().await.unwrap().to_bytes();
    (
        parts.status,
        parts.headers,
        String::from_utf8(bytes.to_vec()).unwrap(),
    )
}

async fn call(
    router: &Router,
    method: Method,
    path: &str,
    language: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let body = body.map(|body| body.to_string());
    let (status, headers, body) = raw(router, method, path, language, body.as_deref()).await;
    (status, headers, serde_json::from_str(&body).unwrap())
}

#[tokio::test]
async fn every_refusal_has_the_one_shape() {
    let (router, _node, _dir) = start().await;

    for (method, path, body, status, refusal) in [
        // No such address, with or without a version.
        (
            Method::GET,
            "/v1/nope",
            None,
            StatusCode::NOT_FOUND,
            json!({ "code": "not_found", "message": "There is nothing at this address." }),
        ),
        (
            Method::POST,
            "/auth/register",
            Some("{}"),
            StatusCode::NOT_FOUND,
            json!({ "code": "not_found", "message": "There is nothing at this address." }),
        ),
        (
            Method::DELETE,
            "/v1/account",
            None,
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "code": "method_not_allowed",
                "message": "This address does not take this method.",
            }),
        ),
        (
            Method::GET,
            "/v1/account",
            None,
            StatusCode::UNAUTHORIZED,
            json!({ "code": "sign_in_required", "message": "Sign in to continue." }),
        ),
        // Not JSON, then JSON of another shape.
        (
            Method::POST,
            "/v1/auth/register",
            Some("{"),
            StatusCode::BAD_REQUEST,
            json!({
                "code": "invalid_request",
                "message": "The request is not one the api understands.",
            }),
        ),
        (
            Method::POST,
            "/v1/auth/register",
            Some(r#"{"email": 1}"#),
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "code": "invalid_request",
                "message": "The request is not one the api understands.",
            }),
        ),
        (
            Method::POST,
            "/v1/auth/password-reset",
            Some(r#"{"email": "nobody", "password": "short", "code": "000000"}"#),
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "code": "validation_failed",
                "message": "Some fields are not valid.",
                "fields": [
                    {
                        "field": "email",
                        "code": "invalid_email",
                        "message": "This is not a valid e-mail address.",
                    },
                    {
                        "field": "password",
                        "code": "weak_password",
                        "message": "The password must be between 8 and 128 characters.",
                        "params": { "min": 8, "max": 128 },
                    },
                ],
            }),
        ),
    ] {
        let (got, headers, answer) = raw(&router, method.clone(), path, None, body).await;
        assert_eq!(got, status, "{method} {path}: {answer}");
        assert_eq!(headers[CONTENT_TYPE], "application/json", "{method} {path}");
        assert_eq!(headers[CONTENT_LANGUAGE], "en", "{method} {path}");
        assert_eq!(
            serde_json::from_str::<Value>(&answer).unwrap(),
            json!({ "error": refusal }),
            "{method} {path}"
        );
    }
}

#[tokio::test]
async fn a_refusal_is_worded_in_the_language_of_the_request() {
    let (router, _node, _dir) = start().await;

    let (status, headers, body) = call(
        &router,
        Method::GET,
        "/v1/account",
        Some("fr-CA,fr;q=0.9,en;q=0.8"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[CONTENT_LANGUAGE], "fr");
    assert_eq!(
        body,
        json!({ "error": {
            "code": "sign_in_required",
            "message": "Connectez-vous pour continuer.",
        } })
    );

    // The fields too; the codes and the bounds stay what they are.
    let (status, headers, body) = call(
        &router,
        Method::POST,
        "/v1/auth/register",
        Some("fr"),
        Some(json!({ "email": "alice@example.com", "password": "short" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(headers[CONTENT_LANGUAGE], "fr");
    assert_eq!(
        body,
        json!({ "error": {
            "code": "validation_failed",
            "message": "Certains champs ne sont pas valides.",
            "fields": [{
                "field": "password",
                "code": "weak_password",
                "message": "Le mot de passe doit contenir entre 8 et 128 caractères.",
                "params": { "min": 8, "max": 128 },
            }],
        } })
    );

    // A language the api does not speak is answered in English, and so is
    // an address nobody serves.
    for (path, language) in [("/v1/account", "de"), ("/nope", "de,fr;q=0")] {
        let (_, headers, body) = call(&router, Method::GET, path, Some(language), None).await;
        assert_eq!(headers[CONTENT_LANGUAGE], "en", "{path}");
        assert!(body["error"]["message"].is_string(), "{body}");
    }
    let (_, headers, body) = call(&router, Method::GET, "/nope", Some("fr"), None).await;
    assert_eq!(headers[CONTENT_LANGUAGE], "fr");
    assert_eq!(body["error"]["message"], "Il n'y a rien à cette adresse.");

    // What is not a refusal says nothing of a language.
    let (status, headers, _) = call(&router, Method::GET, "/health", Some("fr"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!headers.contains_key(CONTENT_LANGUAGE));
}

#[tokio::test]
async fn the_reference_describes_what_the_api_serves() {
    let (router, _node, _dir) = start().await;

    let (status, headers, document) =
        call(&router, Method::GET, "/v1/openapi.json", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[CONTENT_TYPE], "application/json");
    assert_eq!(document["info"]["title"], "imkitchen api");
    assert_eq!(
        document["components"]["securitySchemes"]["session"],
        json!({ "type": "http", "scheme": "bearer" })
    );

    let mut operations: Vec<String> = Vec::new();
    for (path, item) in document["paths"].as_object().unwrap() {
        for (method, operation) in item.as_object().unwrap() {
            operations.push(format!("{} {path}", method.to_uppercase()));
            // Every versioned endpoint refuses with the one shape.
            let refusal = &operation["responses"]["default"]["content"]["application/json"];
            if path.starts_with("/v1/") {
                assert_eq!(
                    refusal["schema"]["$ref"], "#/components/schemas/ErrorBody",
                    "{method} {path}"
                );
                assert!(operation["summary"].is_string(), "{method} {path}");
            } else {
                assert!(refusal.is_null(), "{method} {path}");
            }
        }
    }
    operations.sort();
    assert_eq!(
        operations,
        [
            "DELETE /v1/admin/users/{id}",
            "GET /health",
            "GET /ready",
            "GET /v1/account",
            "GET /v1/admin/users/{id}",
            "POST /v1/account/sign-out-everywhere",
            "POST /v1/admin/users/{id}/sign-out",
            "POST /v1/auth/password-reset",
            "POST /v1/auth/register",
            "POST /v1/auth/sign-in",
            "POST /v1/auth/sign-out",
            "PUT /v1/account/password",
            "PUT /v1/account/profile",
            "PUT /v1/admin/users/{id}/role",
        ]
    );

    // Who must be signed in is said, and what a command takes.
    let paths = &document["paths"];
    assert_eq!(
        paths["/v1/account"]["get"]["security"],
        json!([{ "session": [] }])
    );
    assert!(paths["/v1/auth/register"]["post"]["security"].is_null());
    assert_eq!(
        paths["/v1/account/profile"]["put"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/Profile"
    );
    assert_eq!(
        document["components"]["schemas"]["ErrorBody"]["required"],
        json!(["error"])
    );

    let (status, headers, page) = raw(&router, Method::GET, "/v1/docs", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        headers[CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    assert!(
        page.contains("/v1/auth/register"),
        "the document is in the page"
    );
}
