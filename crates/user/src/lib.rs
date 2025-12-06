pub mod meal_preferences;
pub mod subscription;

mod aggregator;
mod event;
mod types;

pub use aggregator::*;
pub use event::*;
pub use types::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "full")] {
        mod command;
        mod query;

        pub use command::*;
        pub use query::*;
    }
}
