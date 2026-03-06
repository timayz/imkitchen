#[evento::aggregator]
pub enum Subscription {
    LifePremiumToggled { expire_at: u64 },
    StripeCustomerCreated { id: String },
    StripePaymentMethodCreated { id: String },
    StripePaymentIntentCreated { id: String },
    StripePaymentIntentSucceeded { id: String, expire_at: u64 },
}
