//! Runtime configuration.
//!
//! Priority, highest first: `IMKITCHEN_API__<SECTION>__<KEY>` environment
//! variables, the TOML file given with `--config` / `IMKITCHEN_API_CONFIG`,
//! then the defaults set in [`Config::load`]. `config/default.toml` documents
//! the full shape.

use std::fmt;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use config::{ConfigError, Environment, File};
use evento::cluster::Settings;
use imkitchen_aggregates::Sidecar;
use serde::Deserialize;
use timada_identity::RootAccount;

/// The least an `auth.secret` or the token of a sidecar may be, in
/// characters.
const SECRET_MIN_LEN: usize = 32;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    /// This node's place among the api instances that replicate the store
    /// (`node_id`, `peers`, …). Absent: a node alone, talking to no one.
    #[serde(default)]
    pub cluster: Option<Settings>,
    /// Where www and admin read the events from. Absent: no sidecar is
    /// served.
    #[serde(default)]
    pub sidecars: Option<SidecarsConfig>,
    pub auth: AuthConfig,
    pub mail: MailConfig,
    pub images: ImagesConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ImagesConfig {
    /// Where this node keeps the pictures of the recipes: files beside the
    /// store, every node holding its own copy of each.
    pub dir: PathBuf,
    /// Where each node takes the pictures the others copy to it, in the
    /// order of `cluster.peers`, this node included: `http://host:port`, or
    /// `https://` when the nodes talk over TLS (`[cluster.tls]`). Required
    /// with more than one node.
    #[serde(default)]
    pub peers: Vec<String>,
    /// The address to bind for the other nodes. Defaults to this node's
    /// entry in `peers` when that is an address, and is required when it is
    /// a name; `0.0.0.0:<port>` in a container.
    #[serde(default)]
    pub listen: Option<SocketAddr>,
}

impl ImagesConfig {
    /// Where this node takes the other nodes' requests, when it has peers:
    /// `listen`, or the address of its own entry in `peers`.
    pub fn listen_addr(&self, cluster: Option<&Settings>) -> Option<SocketAddr> {
        let cluster = cluster.filter(|cluster| cluster.peers.len() > 1)?;
        self.listen.or_else(|| {
            let own = self.peers.get(usize::try_from(cluster.node_id).ok()?)?;
            let (_, address) = own.split_once("://")?;
            address.trim_end_matches('/').parse().ok()
        })
    }

    fn validate(&self, cluster: Option<&Settings>) -> Result<(), ConfigError> {
        let Some(cluster) = cluster.filter(|cluster| cluster.peers.len() > 1) else {
            return Ok(());
        };
        if self.peers.len() != cluster.peers.len() {
            return Err(ConfigError::Message(format!(
                "images.peers must list where each of the {} nodes of cluster.peers takes pictures, in the same order",
                cluster.peers.len()
            )));
        }
        // A certificate is checked or it is not: the same on every link.
        let scheme = match cluster.tls {
            Some(_) => "https://",
            None => "http://",
        };
        for url in &self.peers {
            if !url.starts_with(scheme) {
                return Err(ConfigError::Message(format!(
                    "images.peers: '{url}' must start with {scheme}: the nodes talk {} [cluster.tls]",
                    if cluster.tls.is_some() {
                        "over the TLS of"
                    } else {
                        "in clear without"
                    }
                )));
            }
        }
        if self.listen_addr(Some(cluster)).is_none() {
            return Err(ConfigError::Message(
                "images.listen is required: this node's entry in images.peers gives no address to bind (0.0.0.0:<port> in a container)"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AuthConfig {
    /// What the one-time codes mailed to prove an address derive from. It
    /// never enters the store, and every node of a cluster holds the same
    /// one. Absent: a node alone draws one each time it starts.
    #[serde(default)]
    pub secret: Option<Secret>,
    /// How long a session lasts once signed in, in seconds. A session in
    /// use past half of it is extended for as long again.
    pub session_lifetime_secs: u64,
    /// The root account: the administrator nobody registers, and the only
    /// one who makes administrators. Absent: there is none.
    #[serde(default)]
    pub root: Option<RootConfig>,
}

impl AuthConfig {
    /// The root account, its address and password hash checked.
    pub fn root_account(&self) -> Result<Option<RootAccount>, ConfigError> {
        self.root
            .as_ref()
            .map(|root| {
                RootAccount::new(&root.email, root.password_hash.expose())
                    .map_err(|err| ConfigError::Message(format!("auth.root: {err}")))
            })
            .transpose()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RootConfig {
    pub email: String,
    /// An argon2 PHC string, as `imkitchen-api hash-password` prints one.
    pub password_hash: Secret,
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
pub struct SidecarsConfig {
    /// Where the sidecars connect: evento's remote protocol, not HTTP.
    pub listen: SocketAddr,
    /// The certificate chain and its key, as PEM files. Absent: connections
    /// are not encrypted, and the tokens travel in clear text.
    #[serde(default)]
    pub tls_cert: Option<PathBuf>,
    #[serde(default)]
    pub tls_key: Option<PathBuf>,
    #[serde(default)]
    pub tokens: SidecarTokens,
}

/// What each sidecar opens its connections with. One without a token is
/// not served.
#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
pub struct SidecarTokens {
    #[serde(default)]
    pub www: Option<Secret>,
    #[serde(default)]
    pub admin: Option<Secret>,
}

impl SidecarsConfig {
    /// The sidecars that are served, each with its token.
    pub fn tokens(&self) -> impl Iterator<Item = (Sidecar, &Secret)> {
        Sidecar::ALL.into_iter().filter_map(|sidecar| {
            let token = match sidecar {
                Sidecar::Www => &self.tokens.www,
                Sidecar::Admin => &self.tokens.admin,
            };
            Some((sidecar, token.as_ref()?))
        })
    }

    /// The certificate and key files, when connections are encrypted.
    pub fn tls(&self) -> Option<(&Path, &Path)> {
        Some((self.tls_cert.as_deref()?, self.tls_key.as_deref()?))
    }

    fn validate(&self) -> Result<(), ConfigError> {
        let mut tokens: Vec<&str> = Vec::new();
        for (sidecar, token) in self.tokens() {
            let name = sidecar.name();
            if token.expose().chars().count() < SECRET_MIN_LEN {
                return Err(ConfigError::Message(format!(
                    "sidecars.tokens.{name} must be at least {SECRET_MIN_LEN} characters"
                )));
            }
            // A token says which sidecar connects.
            if tokens.contains(&token.expose()) {
                return Err(ConfigError::Message(format!(
                    "sidecars.tokens.{name} is the token of another sidecar: each has its own"
                )));
            }
            tokens.push(token.expose());
        }
        if tokens.is_empty() {
            return Err(ConfigError::Message(
                "[sidecars] needs a token under sidecars.tokens: without one nobody may connect"
                    .to_owned(),
            ));
        }
        if self.tls_cert.is_some() != self.tls_key.is_some() {
            return Err(ConfigError::Message(
                "sidecars.tls_cert and sidecars.tls_key go together: set both or neither"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct MailConfig {
    /// The server the one-time codes are mailed through, as a URL:
    /// `smtp://host:port` (plain), `smtp://host:587?tls=required`
    /// (STARTTLS) or `smtps://user:password@host:465` (TLS).
    pub smtp_url: Secret,
    /// Who the mails come from: `name <address>`.
    pub from: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct DatabaseConfig {
    /// This node's store: one Turso file, created by `migrate`. The node
    /// keeps its cluster state in the same directory.
    pub path: PathBuf,
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
pub struct MonitoringConfig {
    /// A `tracing_subscriber::EnvFilter` directive, e.g. `info,tower_http=debug`.
    pub log_level: String,
    pub log_json: bool,
    pub log_target: bool,
    pub log_line_number: bool,
}

impl Config {
    pub fn load(path: Option<&str>) -> Result<Self, ConfigError> {
        let mut builder = config::Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("server.shutdown_timeout_secs", 30)?
            .set_default("database.path", "data/imkitchen.db")?
            .set_default("auth.session_lifetime_secs", 30 * 24 * 3600)?
            .set_default("mail.smtp_url", "smtp://localhost:1025")?
            .set_default("mail.from", "imkitchen <no-reply@imkitchen.localhost>")?
            .set_default("images.dir", "data/images")?
            .set_default("monitoring.log_level", "info")?
            .set_default("monitoring.log_json", false)?
            .set_default("monitoring.log_target", true)?
            .set_default("monitoring.log_line_number", true)?;

        if let Some(path) = path {
            builder = builder.add_source(File::with_name(path).required(true));
        }

        let config: Self = builder
            .add_source(Environment::with_prefix("imkitchen_api").separator("__"))
            .build()?
            .try_deserialize()?;

        if let Some(cluster) = &config.cluster {
            cluster
                .validate()
                .map_err(|err| ConfigError::Message(err.to_string()))?;
        }
        if let Some(sidecars) = &config.sidecars {
            sidecars.validate()?;
        }
        config.images.validate(config.cluster.as_ref())?;
        match &config.auth.secret {
            Some(secret) if secret.expose().chars().count() < SECRET_MIN_LEN => {
                return Err(ConfigError::Message(format!(
                    "auth.secret must be at least {SECRET_MIN_LEN} characters"
                )));
            }
            // A code is mailed by one node and checked by another.
            None if config.cluster.is_some() => {
                return Err(ConfigError::Message(
                    "auth.secret is required with [cluster]: every node derives the codes from it"
                        .to_owned(),
                ));
            }
            _ => {}
        }
        config.auth.root_account()?;

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
        let path = std::env::temp_dir().join(format!("imkitchen-api-{}.toml", std::process::id()));
        std::fs::write(&path, "[server]\nport = 3100\n").unwrap();

        let config = Config::load(path.to_str());
        std::fs::remove_file(&path).unwrap();
        let config = config.unwrap();

        assert_eq!(config.server.port, 3100);
        assert_eq!(config.server.host, "0.0.0.0");
    }

    #[test]
    fn cluster_section_is_read_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cluster.toml");
        let peers = r#"peers = ["127.0.0.1:7000", "127.0.0.1:7001", "127.0.0.1:7002"]"#;
        let peers = format!(
            "{peers}\n[auth]\nsecret = \"{}\"\n[images]\npeers = [{}]",
            "s".repeat(SECRET_MIN_LEN),
            r#""http://127.0.0.1:7100", "http://127.0.0.1:7101", "http://127.0.0.1:7102""#
        );

        std::fs::write(&path, format!("[cluster]\nnode_id = 1\n{peers}\n")).unwrap();
        let cluster = Config::load(path.to_str()).unwrap().cluster.unwrap();
        assert_eq!(cluster.node_id, 1);
        assert_eq!(cluster.peers.len(), 3);

        std::fs::write(&path, format!("[cluster]\nnode_id = 3\n{peers}\n")).unwrap();
        let err = Config::load(path.to_str()).unwrap_err();
        assert!(err.to_string().contains("cluster.node_id"), "{err}");
    }

    #[test]
    fn auth_section_is_read_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.toml");
        let load = |content: &str| {
            std::fs::write(&path, content).unwrap();
            Config::load(path.to_str())
        };
        let secret = "s".repeat(SECRET_MIN_LEN);
        let hash = "$argon2id$v=19$m=19456,t=2,p=1$diXOV+PTSdOCfHj2wDAlMA$iVEeQUJsww/h81mHmrcmcJRWuZs2mY3bopNrchI/yog";

        let config = load(&format!(
            "[auth]\nsecret = \"{secret}\"\n[auth.root]\nemail = \"Root@Example.com\"\npassword_hash = \"{hash}\"\n"
        ))
        .unwrap();
        let root = config.auth.root_account().unwrap().unwrap();
        assert_eq!(root.email(), "root@example.com");
        let printed = format!("{config:?}");
        assert!(!printed.contains(&secret), "{printed}");
        assert!(!printed.contains("argon2"), "{printed}");

        let err = load("[auth]\nsecret = \"short\"\n").unwrap_err();
        assert!(err.to_string().contains("auth.secret"), "{err}");

        let err =
            load("[auth.root]\nemail = \"root@example.com\"\npassword_hash = \"a password\"\n")
                .unwrap_err();
        assert!(err.to_string().contains("auth.root"), "{err}");

        // The nodes of a cluster check each other's codes.
        // One peer: a node alone, which copies its pictures to nobody.
        let cluster = "[cluster]\nnode_id = 0\npeers = [\"127.0.0.1:7000\"]\n";
        let err = load(cluster).unwrap_err();
        assert!(err.to_string().contains("auth.secret"), "{err}");
        assert!(load(&format!("{cluster}[auth]\nsecret = \"{secret}\"\n")).is_ok());
    }

    #[test]
    fn images_section_is_read_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("images.toml");
        let load = |content: &str| {
            std::fs::write(&path, content).unwrap();
            Config::load(path.to_str())
        };
        let cluster = format!(
            "[cluster]\nnode_id = 1\npeers = [\"api-0:7000\", \"api-1:7000\", \"api-2:7000\"]\n\
             listen = \"0.0.0.0:7000\"\n[auth]\nsecret = \"{}\"\n",
            "s".repeat(SECRET_MIN_LEN)
        );

        // A node alone keeps its pictures and asks nobody.
        let alone = Config::load(None).unwrap();
        assert_eq!(alone.images.dir, PathBuf::from("data/images"));
        assert_eq!(alone.images.listen_addr(alone.cluster.as_ref()), None);

        // With peers, every node says where it takes the others' copies.
        let err = load(&cluster).unwrap_err();
        assert!(err.to_string().contains("images.peers"), "{err}");
        let addresses = r#"peers = ["http://127.0.0.1:7100", "http://127.0.0.1:7101", "http://127.0.0.1:7102"]"#;
        let config = load(&format!("{cluster}[images]\n{addresses}\n")).unwrap();
        assert_eq!(
            config.images.listen_addr(config.cluster.as_ref()),
            Some("127.0.0.1:7101".parse().unwrap())
        );

        // A name says where to reach a node, not what to bind.
        let names = r#"peers = ["http://api-0:7100", "http://api-1:7100", "http://api-2:7100"]"#;
        let err = load(&format!("{cluster}[images]\n{names}\n")).unwrap_err();
        assert!(err.to_string().contains("images.listen"), "{err}");
        let config = load(&format!(
            "{cluster}[images]\n{names}\nlisten = \"0.0.0.0:7100\"\n"
        ))
        .unwrap();
        assert_eq!(
            config.images.listen_addr(config.cluster.as_ref()),
            Some("0.0.0.0:7100".parse().unwrap())
        );

        // Without [cluster.tls] the nodes talk in clear, here as there.
        let tls = names.replace("http://", "https://");
        let err = load(&format!(
            "{cluster}[images]\n{tls}\nlisten = \"0.0.0.0:7100\"\n"
        ))
        .unwrap_err();
        assert!(err.to_string().contains("http://"), "{err}");
    }

    #[test]
    fn sidecars_section_is_read_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sidecars.toml");
        let load = |content: &str| {
            std::fs::write(&path, content).unwrap();
            Config::load(path.to_str())
        };
        let listen = "[sidecars]\nlisten = \"127.0.0.1:4000\"\n";
        let www = "w".repeat(SECRET_MIN_LEN);
        let admin = "a".repeat(SECRET_MIN_LEN);

        assert_eq!(Config::load(None).unwrap().sidecars, None);

        let config = load(&format!(
            "{listen}[sidecars.tokens]\nwww = \"{www}\"\nadmin = \"{admin}\"\n"
        ))
        .unwrap();
        let sidecars = config.sidecars.as_ref().unwrap();
        assert_eq!(sidecars.listen.port(), 4000);
        assert_eq!(sidecars.tls(), None);
        let served: Vec<_> = sidecars.tokens().map(|(sidecar, _)| sidecar).collect();
        assert_eq!(served, [Sidecar::Www, Sidecar::Admin]);
        let printed = format!("{config:?}");
        assert!(!printed.contains(&www), "{printed}");

        // Only the sidecars given a token are served.
        let config = load(&format!("{listen}[sidecars.tokens]\nadmin = \"{admin}\"\n")).unwrap();
        let served: Vec<_> = config.sidecars.as_ref().unwrap().tokens().collect();
        assert_eq!(served.len(), 1);
        assert_eq!(served[0].0, Sidecar::Admin);

        let err = load(listen).unwrap_err();
        assert!(err.to_string().contains("sidecars.tokens"), "{err}");

        let err = load(&format!("{listen}[sidecars.tokens]\nwww = \"short\"\n")).unwrap_err();
        assert!(err.to_string().contains("sidecars.tokens.www"), "{err}");

        let err = load(&format!(
            "{listen}[sidecars.tokens]\nwww = \"{www}\"\nadmin = \"{www}\"\n"
        ))
        .unwrap_err();
        assert!(err.to_string().contains("sidecars.tokens.admin"), "{err}");

        let err = load(&format!(
            "{listen}tls_cert = \"tls.crt\"\n[sidecars.tokens]\nwww = \"{www}\"\n"
        ))
        .unwrap_err();
        assert!(err.to_string().contains("sidecars.tls_key"), "{err}");
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(Config::load(Some("does/not/exist.toml")).is_err());
    }
}
