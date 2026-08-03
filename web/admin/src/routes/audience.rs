use axum::extract::State;
use axum::response::IntoResponse;
use imkitchen_audience::daily_stat::{BreakdownDim, BreakdownRow, DayTotal, day_string};

use imkitchen_web_shared::{
    AppState,
    auth::AuthAdmin,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "admin-audience.html")]
pub struct AudienceTemplate {
    pub current_path: String,
    pub enabled: bool,
    pub today: u32,
    pub last7: u32,
    pub last30: u32,
    pub per_day: Vec<DayTotal>,
    pub paths: Vec<BreakdownRow>,
    pub devices: Vec<BreakdownRow>,
    pub browsers: Vec<BreakdownRow>,
    pub countries: Vec<BreakdownRow>,
    pub referrers: Vec<BreakdownRow>,
}

impl Default for AudienceTemplate {
    fn default() -> Self {
        Self {
            current_path: "audience".to_owned(),
            enabled: false,
            today: 0,
            last7: 0,
            last30: 0,
            per_day: Vec::new(),
            paths: Vec::new(),
            devices: Vec::new(),
            browsers: Vec::new(),
            countries: Vec::new(),
            referrers: Vec::new(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    State(app): State<AppState>,
    user: AuthAdmin,
) -> impl IntoResponse {
    let Some(audience) = app.audience.as_ref() else {
        return template.render(AudienceTemplate::default());
    };

    let now = time::UtcDateTime::now().unix_timestamp() as u64;
    let (today_day, from_7, from_30) = match (
        day_string(now),
        day_string(now - 6 * 86400),
        day_string(now - 29 * 86400),
    ) {
        (Ok(a), Ok(b), Ok(c)) => (a, b, c),
        _ => return template.render(AudienceTemplate::default()),
    };

    let today =
        imkitchen_web_shared::try_page_response!(audience.total_since(&today_day), template);
    let last7 = imkitchen_web_shared::try_page_response!(audience.total_since(&from_7), template);
    let last30 = imkitchen_web_shared::try_page_response!(audience.total_since(&from_30), template);
    let per_day = imkitchen_web_shared::try_page_response!(audience.per_day(&from_30), template);
    let paths = imkitchen_web_shared::try_page_response!(
        audience.breakdown(BreakdownDim::Path, &from_30, 10),
        template
    );
    let devices = imkitchen_web_shared::try_page_response!(
        audience.breakdown(BreakdownDim::Device, &from_30, 10),
        template
    );
    let browsers = imkitchen_web_shared::try_page_response!(
        audience.breakdown(BreakdownDim::Browser, &from_30, 10),
        template
    );
    let countries = imkitchen_web_shared::try_page_response!(
        audience.breakdown(BreakdownDim::Country, &from_30, 10),
        template
    );
    let referrers =
        imkitchen_web_shared::try_page_response!(audience.recent_referrers(&from_30, 10), template);

    template.render(AudienceTemplate {
        enabled: true,
        today,
        last7,
        last30,
        per_day,
        paths,
        devices,
        browsers,
        countries,
        referrers,
        ..Default::default()
    })
}
