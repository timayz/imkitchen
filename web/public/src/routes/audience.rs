use axum::body::Bytes;
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
    // Bytes rather than Form/RawForm: those reject unexpected content types
    // with a 4xx, and this endpoint must always no-op successfully.
    body: Bytes,
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
        // The landing page's document.referrer, sent by twinspark via
        // ts-data. The beacon request's own Referer header is useless here —
        // it is the landing page URL itself, not the traffic source.
        let referrer = url::form_urlencoded::parse(&body)
            .find(|(key, _)| key == "referrer")
            .map(|(_, value)| value.into_owned());

        if let Err(err) = audience
            .record_visit(RecordVisitInput {
                path,
                user_agent: user_agent.to_string(),
                timezone,
                referrer,
            })
            .await
        {
            tracing::error!("failed to record audience visit: {err}");
        }
    }

    ([("ts-swap", "skip")], "")
}
