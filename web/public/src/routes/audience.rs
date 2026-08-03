use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use axum_extra::headers::UserAgent;
use imkitchen_audience::RecordVisitInput;

use imkitchen_web_shared::AppState;

/// Landing-page visit beacon, fired by twinspark after page load. Always
/// responds with an empty `ts-swap: skip` no-op: recording failures must never
/// surface to the visitor.
pub async fn visit(
    State(app): State<AppState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Some(audience) = app.audience.as_ref() {
        let timezone = headers
            .get("TS-Timezone")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("UTC")
            .to_owned();
        let path = headers
            .get("TS-URL")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("/")
            .to_owned();

        if let Err(err) = audience
            .record_visit(RecordVisitInput {
                path,
                user_agent: user_agent.to_string(),
                timezone,
            })
            .await
        {
            tracing::error!("failed to record audience visit: {err}");
        }
    }

    ([("ts-swap", "skip")], "")
}
