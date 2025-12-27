// pub mod meal_preferences;
// pub mod reset_password;
// pub mod subscription;

mod aggregator;
mod value_object;

pub use aggregator::*;
pub use value_object::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "full")] {
        mod command;
        mod query;

        pub use command::*;
        pub use query::*;
    }
}
