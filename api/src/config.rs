//! Runtime configuration.
//!
//! Priority, highest first: `IMKITCHEN_API__<SECTION>__<KEY>` environment
//! variables, the TOML file given with `--config` / `IMKITCHEN_API_CONFIG`,
//! then the defaults set in [`Config::load`]. `config/default.toml` documents
//! the full shape.

use std::path::PathBuf;
use std::time::Duration;

use config::{ConfigError, Environment, File};
use evento::cluster::Settings;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    /// This node's place among the api instances that replicate the store
    /// (`node_id`, `peers`, …). Absent: a node alone, talking to no one.
    #[serde(default)]
    pub cluster: Option<Settings>,
    pub monitoring: MonitoringConfig,
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

        std::fs::write(&path, format!("[cluster]\nnode_id = 1\n{peers}\n")).unwrap();
        let cluster = Config::load(path.to_str()).unwrap().cluster.unwrap();
        assert_eq!(cluster.node_id, 1);
        assert_eq!(cluster.peers.len(), 3);

        std::fs::write(&path, format!("[cluster]\nnode_id = 3\n{peers}\n")).unwrap();
        let err = Config::load(path.to_str()).unwrap_err();
        assert!(err.to_string().contains("cluster.node_id"), "{err}");
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(Config::load(Some("does/not/exist.toml")).is_err());
    }
}
