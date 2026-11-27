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
use topcoat::context::BaseUrl;

/// The least the token of app may be, in characters: what the api asks of
/// `sidecars.tokens.app`.
const TOKEN_MIN_LEN: usize = 32;

/// The least `session.key` may be, in bytes: what a key is derived from.
const SESSION_KEY_MIN_LEN: usize = 32;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    pub www: WwwConfig,
    #[serde(default)]
    pub session: SessionConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// The address users reach app at. Over `https`, the cookies of a
    /// session are only ever sent encrypted.
    pub url: String,
    /// How long requests in flight get to finish once the server was asked
    /// to stop, in seconds.
    pub shutdown_timeout_secs: u64,
}

impl ServerConfig {
    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.shutdown_timeout_secs)
    }

    /// Whether app is reached over HTTPS.
    pub fn secure(&self) -> bool {
        self.url.starts_with("https://")
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if !self.url.starts_with("http://") && !self.secure() {
            return Err(ConfigError::Message(format!(
                "server.url must be an http(s) address, not '{}'",
                self.url
            )));
        }
        Ok(())
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
    /// Where the api answers HTTP: where users register, sign in and out,
    /// are asked about on every page, and what they do is sent as commands.
    pub url: String,
    /// Where the browsers of the users reach the api: what the pictures of
    /// the recipes are shown from. Behind the gateway, `/api` of the main
    /// domain.
    pub public_url: String,
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
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err(ConfigError::Message(format!(
                "api.url must be an http(s) address, not '{}'",
                self.url
            )));
        }
        if !self.public_url.starts_with("http://") && !self.public_url.starts_with("https://") {
            return Err(ConfigError::Message(format!(
                "api.public_url must be an http(s) address, not '{}'",
                self.public_url
            )));
        }
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

/// Where the public pages of imkitchen are.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct WwwConfig {
    /// The address that always serves www (`about.`), without a path of its
    /// pages: the main domain serves a signed-in user app.
    pub url: String,
}

impl WwwConfig {
    /// `url`, for the pages to link to those of www.
    pub fn base_url(&self) -> Result<BaseUrl, ConfigError> {
        BaseUrl::new(&self.url).map_err(|err| {
            ConfigError::Message(format!(
                "www.url must be the address of www, not '{}': {err}",
                self.url
            ))
        })
    }
}

/// What the sessions of the users are kept with.
#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
pub struct SessionConfig {
    /// What the cookie of a session is encrypted under: 32 bytes or more,
    /// the same for every replica of app. Absent: one is drawn each time
    /// app starts, and whoever was signed in signs in again.
    #[serde(default)]
    pub key: Option<Secret>,
}

impl SessionConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if let Some(key) = &self.key
            && key.expose().len() < SESSION_KEY_MIN_LEN
        {
            return Err(ConfigError::Message(format!(
                "session.key must be at least {SESSION_KEY_MIN_LEN} bytes"
            )));
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
            .set_default("server.url", "http://localhost:3030")?
            .set_default("server.shutdown_timeout_secs", 30)?
            .set_default("database.path", "data/app.db")?
            .set_default("api.url", "http://127.0.0.1:3000")?
            .set_default("api.public_url", "http://localhost:3000")?
            .set_default("api.events", "127.0.0.1:4000")?
            .set_default("www.url", "http://localhost:3010")?
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

        config.server.validate()?;
        config.api.validate()?;
        config.www.base_url()?;
        config.session.validate()?;

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

        let config = load("[api]\nurl = \"http://api.imkitchen.svc:3000\"\n").unwrap();
        assert_eq!(config.api.url, "http://api.imkitchen.svc:3000");

        let err = load("[api]\nurl = \"api:3000\"\n").unwrap_err();
        assert!(err.to_string().contains("api.url"), "{err}");

        let config = load("[api]\npublic_url = \"https://imkitchen.app/api\"\n").unwrap();
        assert_eq!(config.api.public_url, "https://imkitchen.app/api");

        let err = load("[api]\npublic_url = \"/api\"\n").unwrap_err();
        assert!(err.to_string().contains("api.public_url"), "{err}");
    }

    #[test]
    fn sessions_are_configured_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.toml");
        let load = |content: &str| {
            std::fs::write(&path, content).unwrap();
            Config::load(path.to_str())
        };
        let key = "k".repeat(SESSION_KEY_MIN_LEN);

        let config = Config::load(None).unwrap();
        assert_eq!(config.session.key, None);
        assert!(!config.server.secure());

        let config = load(&format!(
            "[server]\nurl = \"https://imkitchen.app\"\n[session]\nkey = \"{key}\"\n"
        ))
        .unwrap();
        assert!(config.server.secure());
        assert_eq!(config.session.key.as_ref().unwrap().expose(), key);
        let printed = format!("{config:?}");
        assert!(!printed.contains(&key), "{printed}");

        let err = load("[session]\nkey = \"short\"\n").unwrap_err();
        assert!(err.to_string().contains("session.key"), "{err}");

        let err = load("[server]\nurl = \"imkitchen.app\"\n").unwrap_err();
        assert!(err.to_string().contains("server.url"), "{err}");
    }

    #[test]
    fn the_address_of_www_is_read_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("www.toml");
        let load = |content: &str| {
            std::fs::write(&path, content).unwrap();
            Config::load(path.to_str())
        };

        let config = Config::load(None).unwrap();
        assert_eq!(
            config.www.base_url().unwrap().as_str(),
            "http://localhost:3010"
        );

        let config = load("[www]\nurl = \"https://about.imkitchen.app/\"\n").unwrap();
        assert_eq!(
            config.www.base_url().unwrap().as_str(),
            "https://about.imkitchen.app"
        );

        let err = load("[www]\nurl = \"about.imkitchen.app\"\n").unwrap_err();
        assert!(err.to_string().contains("www.url"), "{err}");
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(Config::load(Some("does/not/exist.toml")).is_err());
    }
}
