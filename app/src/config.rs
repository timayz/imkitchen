//! Runtime configuration.
//!
//! Priority, highest first: `IMKITCHEN_APP__<SECTION>__<KEY>` environment
//! variables, the TOML file given with `--config` / `IMKITCHEN_APP_CONFIG`,
//! then the defaults set in [`Config::load`]. `config/default.toml` documents
//! the full shape.

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use config::{ConfigError, Environment, File};
use serde::Deserialize;

/// The least the token of app may be, in characters: what the api asks of
/// `sidecars.tokens.app`.
const TOKEN_MIN_LEN: usize = 32;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// How long requests in flight get to finish once the server was asked
    /// to stop, in seconds.
    pub shutdown_timeout_secs: u64,
}

impl ServerConfig {
    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.shutdown_timeout_secs)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct DatabaseConfig {
    /// The store of app: one Turso file, created by `migrate`, holding its
    /// read models and the cursors of the subscriptions that fill them.
    pub path: PathBuf,
}

/// How app reaches the api.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ApiConfig {
    /// The api's listener for the sidecars (`[sidecars]` there): an IP and
    /// port, or a `host:port` looked up again on every reconnect.
    pub events: String,
    /// What app opens its connections with: `sidecars.tokens.app` of the
    /// api. Absent: app does not serve.
    #[serde(default)]
    pub token: Option<Secret>,
    /// The certificate the listener's is checked against, as a PEM file.
    /// Absent: the connection is not encrypted, and the token travels in
    /// clear text.
    #[serde(default)]
    pub tls_ca: Option<PathBuf>,
    /// The name the listener's certificate must carry. Absent: the host of
    /// `events`.
    #[serde(default)]
    pub tls_server_name: Option<String>,
}

impl ApiConfig {
    /// The host of `events`, without the brackets of an IPv6 address.
    pub fn events_host(&self) -> Option<&str> {
        let (host, _) = self.events.rsplit_once(':')?;
        Some(host.trim_matches(['[', ']'])).filter(|host| !host.is_empty())
    }

    /// The token, for a connection that reaches the api.
    pub fn token(&self) -> Result<&Secret, ConfigError> {
        self.token.as_ref().ok_or_else(|| {
            ConfigError::Message(
                "api.token is required: the token the api gives app (sidecars.tokens.app)"
                    .to_owned(),
            )
        })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.events_host().is_none() {
            return Err(ConfigError::Message(format!(
                "api.events must be host:port, not '{}'",
                self.events
            )));
        }
        if let Some(token) = &self.token
            && token.expose().chars().count() < TOKEN_MIN_LEN
        {
            return Err(ConfigError::Message(format!(
                "api.token must be at least {TOKEN_MIN_LEN} characters"
            )));
        }
        if self.tls_server_name.is_some() && self.tls_ca.is_none() {
            return Err(ConfigError::Message(
                "api.tls_server_name needs api.tls_ca: without it the connection is not encrypted"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

/// A value of the configuration that is never printed.
#[derive(Deserialize, Clone, PartialEq)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("******")
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct MonitoringConfig {
    /// A `tracing_subscriber::EnvFilter` directive, e.g. `info,evento=debug`.
    pub log_level: String,
    pub log_json: bool,
    pub log_target: bool,
    pub log_line_number: bool,
}

impl Config {
    pub fn load(path: Option<&str>) -> Result<Self, ConfigError> {
        let mut builder = config::Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3030)?
            .set_default("server.shutdown_timeout_secs", 30)?
            .set_default("database.path", "data/app.db")?
            .set_default("api.events", "127.0.0.1:4000")?
            .set_default("monitoring.log_level", "info")?
            .set_default("monitoring.log_json", false)?
            .set_default("monitoring.log_target", true)?
            .set_default("monitoring.log_line_number", true)?;

        if let Some(path) = path {
            builder = builder.add_source(File::with_name(path).required(true));
        }

        let config: Self = builder
            .add_source(Environment::with_prefix("imkitchen_app").separator("__"))
            .build()?
            .try_deserialize()?;

        config.api.validate()?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_file_documents_the_built_in_defaults() {
        let documented = concat!(env!("CARGO_MANIFEST_DIR"), "/config/default.toml");

        assert_eq!(
            Config::load(Some(documented)).unwrap(),
            Config::load(None).unwrap()
        );
    }

    #[test]
    fn file_overrides_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.toml");
        std::fs::write(&path, "[server]\nport = 3130\n").unwrap();

        let config = Config::load(path.to_str()).unwrap();

        assert_eq!(config.server.port, 3130);
        assert_eq!(config.server.host, "0.0.0.0");
    }

    #[test]
    fn api_section_is_read_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api.toml");
        let load = |content: &str| {
            std::fs::write(&path, content).unwrap();
            Config::load(path.to_str())
        };
        let token = "t".repeat(TOKEN_MIN_LEN);

        // Enough to migrate; serving asks for the token.
        let config = Config::load(None).unwrap();
        assert_eq!(config.api.events_host(), Some("127.0.0.1"));
        assert!(config.api.token().is_err());

        let config = load(&format!(
            "[api]\nevents = \"api.imkitchen.svc:4000\"\ntoken = \"{token}\"\n\
             tls_ca = \"ca.crt\"\n"
        ))
        .unwrap();
        assert_eq!(config.api.events_host(), Some("api.imkitchen.svc"));
        assert_eq!(config.api.token().unwrap().expose(), token);
        let printed = format!("{config:?}");
        assert!(!printed.contains(&token), "{printed}");

        let config = load("[api]\nevents = \"[::1]:4000\"\n").unwrap();
        assert_eq!(config.api.events_host(), Some("::1"));

        let err = load("[api]\nevents = \"api\"\n").unwrap_err();
        assert!(err.to_string().contains("api.events"), "{err}");

        let err = load("[api]\ntoken = \"short\"\n").unwrap_err();
        assert!(err.to_string().contains("api.token"), "{err}");

        let err = load("[api]\ntls_server_name = \"api\"\n").unwrap_err();
        assert!(err.to_string().contains("api.tls_ca"), "{err}");
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(Config::load(Some("does/not/exist.toml")).is_err());
    }
}
