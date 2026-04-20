use bitcode::{Decode, Encode};
use serde::Deserialize;

use crate::user::subscription::{Address, PaymentDetails};

#[derive(Default, Encode, Decode, Clone, Deserialize, Debug, PartialEq)]
pub struct InvoiceAddress {
    pub name: String,
    pub email: String,
    pub address: Address,
}

#[evento::aggregator]
pub enum Invoice {
    Created {
        key: String,
        number: u64,
        payment_method_id: String,
        from: InvoiceAddress,
        to: InvoiceAddress,
        paid_at: u64,
        expire_at: u64,
        details: PaymentDetails,
    },
}
