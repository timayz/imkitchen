//! The groceries: what a user has to buy to cook their list. What there is
//! to buy is no event: it is worked out from the list and its recipes as
//! they stand ([`items`]). The events are what the user says of it, item by
//! item.

pub mod items;

/// What one user says of their groceries, under the id of the user. A user
/// who never said anything has no stream.
///
/// An item is named by `item_id`, which says what it is of and in what
/// unit, whatever the list. What is said of it is said of the amount the
/// list then called for (`needed`, `None` when its recipes give none), and
/// holds for as long as the list calls for that much.
#[evento::aggregate(name = "imkitchen/Groceries")]
pub enum Groceries {
    /// The item is in the basket.
    Checked {
        item_id: String,
        needed: Option<f64>,
    },
    /// The item is to buy again.
    Unchecked { item_id: String },
    /// The user buys `quantity` of the item rather than what the list
    /// calls for; `None` for what the list calls for again.
    Adjusted {
        item_id: String,
        needed: Option<f64>,
        quantity: Option<f64>,
    },
}
