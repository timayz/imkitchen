pub mod invoice;
pub mod invoice_user;
mod scheduler;
pub mod subscription;

pub use scheduler::{scheduler, shed_subscription};

use evento::Executor;

#[derive(Clone)]
pub struct Billing<E: Executor> {
    pub subscription: subscription::Module<E>,
    pub invoice: invoice_user::Module<E>,
}

impl<E: Executor> Billing<E> {
    pub fn new(state: imkitchen_shared::State<E>) -> Self
    where
        imkitchen_shared::State<E>: Clone,
    {
        Self {
            subscription: subscription::Module(state.clone()),
            invoice: invoice_user::Module(state),
        }
    }
}
