use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use evento::Evento;

#[derive(Clone)]
pub struct AppState {
    pub inner: imkitchen_core::State<Evento>,
    pub config: crate::config::Config,
    pub stripe: stripe::Client,
    pub identity: imkitchen_identity::Module<Evento>,
    pub billing: imkitchen_billing::Billing<Evento>,
    pub core: imkitchen_core::Core<Evento>,
    /// Present only when the `[audience]` config section is set; runs on its
    /// own evento instance/database.
    pub audience: Option<imkitchen_audience::Module<Evento>>,
    pub import_jobs: AdminImportJobs,
    pub sitemap: SitemapStore,
}

impl Deref for AppState {
    type Target = imkitchen_core::State<Evento>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// One error encountered while processing an admin batch import. `scope` is either
/// `"author"` (the whole author folder was skipped) or `"recipe"` (a single recipe failed).
#[derive(Clone, Default)]
pub struct AdminImportError {
    pub scope: String,
    pub name: String,
    pub message: String,
}

/// Progress / result of a single admin batch-import job, tracked in memory while the
/// background task runs and read back by the status-polling endpoint.
#[derive(Clone, Default)]
pub struct AdminImportProgress {
    pub done: bool,
    pub authors_total: usize,
    /// Total recipe entries discovered in the ZIP (drives the progress bar denominator).
    pub recipes_total: usize,
    /// Attempts completed so far (success + failure); drives the progress bar numerator.
    pub recipes_processed: usize,
    /// Successfully imported recipes; shown in the final summary.
    pub recipes_imported: usize,
    pub errors: Vec<AdminImportError>,
}

/// In-memory registry of running/completed import jobs, keyed by job id.
pub type AdminImportJobs = Arc<Mutex<HashMap<String, AdminImportProgress>>>;

/// One fully-built sitemap in the encodings we serve, so requests never
/// compress. Swapped in whole by the background rebuild task; all fields are
/// empty only before the first successful build.
#[derive(Default)]
pub struct SitemapPayload {
    pub identity: axum::body::Bytes,
    pub gzip: axum::body::Bytes,
    pub brotli: axum::body::Bytes,
}

/// Always-warm pre-rendered sitemap. Readers clone an `Arc` under a momentary
/// read lock; the rebuild task replaces the whole payload. In-memory only, so
/// this assumes a single server process (as does the evento setup).
#[derive(Clone, Default)]
pub struct SitemapStore(Arc<std::sync::RwLock<Arc<SitemapPayload>>>);

impl SitemapStore {
    pub fn load(&self) -> Arc<SitemapPayload> {
        self.0.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn store(&self, payload: SitemapPayload) {
        *self.0.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(payload);
    }
}
