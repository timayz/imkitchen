//! Where www reads the api's events from: the api's listener for the
//! sidecars, followed as an evento `Sidecar`.
//!
//! The events come from the api, over evento's remote protocol; the cursors
//! of the subscriptions stay in www's own store. The api serves www the
//! aggregates `imkitchen_aggregates::Sidecar::Www` reads and takes nothing
//! from it: www writes no event, a command goes to the api over HTTP.
//!
//! The client is there whether the api answers or not: www starts, serves
//! its pages from what its store holds, and its subscriptions go on from
//! their cursors once the api is reached, at first or again.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use evento::RemoteClient;
use evento::remote::ClientState;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName};
use tokio_rustls::rustls::{self, ClientConfig, RootCertStore};

use crate::config::ApiConfig;
use crate::db::Store;

/// What the subscriptions of www run over: the api's events, the store of
/// www for their cursors.
pub type Executor = evento::Sidecar<RemoteClient, Store>;

/// The connection of www to the api's events.
#[derive(Clone)]
pub struct Events {
    client: RemoteClient,
    executor: Executor,
}

impl Events {
    /// Follows the api `config` names, the cursors kept in `store`. Returns
    /// at once: the api is dialled in the background, for as long as www
    /// runs. Must be called within a Tokio runtime.
    pub fn connect(config: &ApiConfig, store: Store) -> Result<Self> {
        let mut client = RemoteClient::builder(config.events.as_str())
            .token(config.token()?.expose().to_owned());
        if let Some(ca) = &config.tls_ca {
            let server_name = config
                .tls_server_name
                .as_deref()
                .or(config.events_host())
                .context("api.events names no host")?;
            let (connector, server_name) = tls(ca, server_name)?;
            client = client.tls(connector, server_name);
        } else {
            tracing::warn!(
                api = %config.events,
                "following the api without encryption: the token travels in clear text"
            );
        }
        let client = client.connect_lazy();
        Ok(Self {
            executor: evento::Sidecar::new(client.clone(), store),
            client,
        })
    }

    /// Waits for the api to answer, `patience` at most, and says whether it
    /// did. An api that does not know the token is an error: it never will,
    /// and the token is to be set right.
    pub async fn reached(&self, patience: Duration) -> Result<bool> {
        match tokio::time::timeout(patience, self.client.connected()).await {
            Ok(connected) => connected
                .map(|()| true)
                .context("the api refuses api.token: it is not its sidecars.tokens.www"),
            Err(_) => Ok(false),
        }
    }

    /// What subscriptions are started over.
    pub fn executor(&self) -> &Executor {
        &self.executor
    }

    /// How www stands with the api, in a word: `connected`, `connecting`
    /// while the api is away, `refused` for good when it does not know the
    /// token.
    pub fn state(&self) -> &'static str {
        match self.client.state() {
            ClientState::Connected => "connected",
            ClientState::Refused => "refused",
            _ => "connecting",
        }
    }
}

/// The client side of TLS: the certificates of the PEM file `ca` are the
/// only ones trusted, for a listener answering to `server_name`.
fn tls(ca: &Path, server_name: &str) -> Result<(TlsConnector, ServerName<'static>)> {
    // The process's provider when something installed one.
    let provider = CryptoProvider::get_default()
        .cloned()
        .unwrap_or_else(|| Arc::new(rustls::crypto::aws_lc_rs::default_provider()));
    let mut roots = RootCertStore::empty();
    for cert in CertificateDer::pem_file_iter(ca)
        .with_context(|| format!("cannot read {}", ca.display()))?
    {
        roots
            .add(cert.with_context(|| format!("cannot parse {}", ca.display()))?)
            .with_context(|| {
                format!(
                    "{} holds a certificate that cannot be trusted",
                    ca.display()
                )
            })?;
    }
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .context("the TLS provider supports no protocol version")?
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name = ServerName::try_from(server_name.to_owned())
        .with_context(|| format!("{server_name:?} is not a name a certificate can carry"))?;
    Ok((TlsConnector::from(Arc::new(config)), server_name))
}
