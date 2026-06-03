use std::ops::Deref;

use evento::Evento;

#[derive(Clone)]
pub struct AppState {
    pub inner: imkitchen_core::State<Evento>,
    pub config: crate::config::Config,
    pub stripe: stripe::Client,
    pub identity: imkitchen_identity::Module<Evento>,
    pub billing: imkitchen_billing::Billing<Evento>,
    pub core: imkitchen_core::Core<Evento>,
}

impl Deref for AppState {
    type Target = imkitchen_core::State<Evento>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
