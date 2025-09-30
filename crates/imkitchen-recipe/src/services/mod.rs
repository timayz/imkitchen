pub mod search;
pub mod popularity;
pub mod discovery_data;
pub mod recommendation;

pub use search::*;
pub use popularity::{PopularityService, PopularityConfig, TimeWindow};
pub use discovery_data::{
    DiscoveryDataService, RecipeDiscoveryMetrics, 
    PreferenceType, DiscoveryQueryAnalytics, RecipeDiscoveryEvent, DiscoveryDataError
};
pub use recommendation::*;