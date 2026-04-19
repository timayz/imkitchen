use crate::user::subscription::{Address, PaymentDetails};

#[evento::aggregator]
pub enum Invoice {
    Created {
        key: String,
        number: u64,
        payment_method_id: String,
        name: Option<String>,
        address: Option<Address>,
        paid_at: u64,
        expire_at: u64,
        details: PaymentDetails,
    },
}
