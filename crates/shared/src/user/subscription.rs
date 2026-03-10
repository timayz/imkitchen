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
    },
    StripePaymentIntentSucceeded {
        id: String,
        payment_method_id: String,
        plan: String,
        country: String,
        state: String,
        expire_at: u64,
    },
    Cancelled,
}
