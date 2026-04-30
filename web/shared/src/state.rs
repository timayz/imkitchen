use std::ops::Deref;

use evento::sql::RwSqlite;

#[derive(Clone)]
pub struct AppState {
    pub inner: imkitchen_core::State<RwSqlite>,
    pub config: crate::config::Config,
    pub stripe: stripe::Client,
    pub identity: imkitchen_identity::Module<RwSqlite>,
    pub billing: imkitchen_billing::Billing<RwSqlite>,
    pub core: imkitchen_core::Core<RwSqlite>,
}

impl Deref for AppState {
    type Target = imkitchen_core::State<RwSqlite>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
