/// One aggregate per visit (`evento::create()` ULID). The visit time comes
/// from evento's event metadata, so it is not duplicated in the payload.
#[evento::aggregate]
pub enum PageVisit {
    Visited {
        /// Normalized to a closed set ("/", "/about", "other") to bound
        /// rollup cardinality on an unauthenticated endpoint.
        path: String,
        /// woothee category: "pc", "smartphone", "mobilephone", ...
        device: String,
        browser: String,
        os: String,
        /// ISO 3166-1 alpha-2, "ZZ" when the timezone is unknown.
        country: String,
        timezone: String,
        /// Referrer host (e.g. "google.com"), "direct" when there is none.
        referrer: String,
    },
}
