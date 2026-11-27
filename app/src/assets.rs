//! The assets of app (its stylesheet), at addresses of app's own.
//!
//! topcoat serves the assets of an application under `/_topcoat/assets`,
//! whichever application it is: on the main domain, which the gateway
//! shares between www and app, a visitor who is not signed in would ask
//! www for the stylesheet of the pages of `/auth`. app serves its own under
//! [`PREFIX`] instead, which the gateway routes to app whoever asks.

use std::collections::HashSet;
use std::path::Path as Dir;

use topcoat::{
    asset::{AssetBundle, AssetConfig, AssetRoute, BundledAsset, RouterBuilderAssetExt},
    context::Cx,
    router::{Body, Methods, Path, PathBuf, Route, RouteFuture, RouteId, RouterBuilder},
};

/// What the address of every asset of app starts with.
pub const PREFIX: &str = "/_app/assets";

/// Gives `router` the assets of `bundle`: each is served under [`PREFIX`],
/// and that is where the pages link to them.
pub fn serve(mut router: RouterBuilder, bundle: AssetBundle) -> RouterBuilder {
    let mut served = HashSet::new();
    for asset in bundle.catalog().assets() {
        if served.insert(asset.name()) {
            router = router.route(Moved::new(bundle.dir(), asset));
        }
    }
    router.assets(AssetConfig::hosted_at(PREFIX, bundle))
}

/// An asset as topcoat serves it, under [`PREFIX`].
struct Moved {
    path: PathBuf,
    served: AssetRoute,
}

impl Moved {
    fn new(dir: &Dir, asset: &BundledAsset) -> Self {
        let served = AssetRoute::new(dir, asset);
        // The name of the file, as topcoat writes it in an address.
        let name = served
            .path()
            .as_str()
            .rsplit('/')
            .next()
            .unwrap_or_default();
        Self {
            path: Path::new(&format!("{PREFIX}/{name}")).to_owned(),
            served,
        }
    }
}

impl Route for Moved {
    fn id(&self) -> RouteId {
        self.served.id()
    }

    fn methods(&self) -> Methods<'_> {
        self.served.methods()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn handle<'cx>(&'cx self, cx: &'cx Cx, body: Body) -> RouteFuture<'cx> {
        self.served.handle(cx, body)
    }
}
