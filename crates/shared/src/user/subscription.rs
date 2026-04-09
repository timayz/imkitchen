use bitcode::{Decode, Encode};
use serde::Deserialize;

#[derive(Default, Encode, Decode, Clone, Deserialize, Debug, PartialEq)]
pub struct PaymentDetails {
    pub plan: String,
    pub price: u32,
    pub tax: u32,
    pub tax_rate: Option<f64>,
}

#[derive(Encode, Decode, Clone, Deserialize, Debug, PartialEq)]
pub struct Address {
    /// City, district, suburb, town, or village.
    pub city: Option<String>,
    /// Two-letter country code ([ISO 3166-1 alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)).
    pub country: Option<String>,
    /// Address line 1, such as the street, PO Box, or company name.
    pub line1: Option<String>,
    /// Address line 2, such as the apartment, suite, unit, or building.
    pub line2: Option<String>,
    /// ZIP or postal code.
    pub postal_code: Option<String>,
    /// State, county, province, or region ([ISO 3166-2](https://en.wikipedia.org/wiki/ISO_3166-2)).
    pub state: Option<String>,
}

impl From<stripe_shared::Address> for Address {
    fn from(value: stripe_shared::Address) -> Self {
        Self {
            city: value.city,
            country: value.country,
            line1: value.line1,
            line2: value.line2,
            postal_code: value.postal_code,
            state: value.state,
        }
    }
}

#[evento::aggregator]
pub enum Subscription {
    LifePremiumToggled {
        expire_at: u64,
    },
    StripeCustomerCreated {
        id: String,
    },
    StripePaymentIntentCreated {
        id: String,
        details: PaymentDetails,
    },
    StripeSetupIntentCreated {
        id: String,
    },
    StripePaymentIntentSucceeded {
        id: String,
        payment_method_id: String,
        name: Option<String>,
        address: Option<Address>,
        expire_at: u64,
        details: PaymentDetails,
    },
    StripeSetupIntentSucceeded {
        id: String,
        payment_method_id: String,
        name: Option<String>,
        address: Option<Address>,
    },
    Cancelled,
}
