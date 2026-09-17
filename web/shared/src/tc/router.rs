use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Router, RouterBuilder},
    runtime::RouterBuilderRuntimeExt,
};

use crate::AppState;

/// Base router every web crate registers its topcoat routes on.
///
/// The asset bundle (which carries the browser runtime script) is optional so
/// router tests can run without one; pages then render without interactivity.
pub fn builder(state: AppState) -> RouterBuilder {
    let builder = super::pages::routes(Router::builder().runtime().cookies().app_context(state));

    match AssetBundle::load() {
        Ok(bundle) => builder.assets(bundle),
        Err(err) => {
            tracing::warn!("topcoat asset bundle not loaded, pages will not be interactive: {err}");
            builder
        }
    }
}
