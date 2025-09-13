use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub jwt: JwtSettings,
    pub app: AppSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtSettings {
    pub secret: String,
    pub expires_in: u64, // seconds
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    pub environment: Environment,
    pub upload_path: String,
    pub max_file_size: u64, // bytes
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub level: String,
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        let _ = dotenvy::dotenv();

        let settings = Settings {
            server: ServerSettings {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .map_err(|_| ConfigError::InvalidPort)?,
            },
            database: DatabaseSettings {
                url: env::var("DATABASE_URL")
                    .map_err(|_| ConfigError::MissingDatabaseUrl)?,
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()
                    .unwrap_or(20),
                min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
            redis: RedisSettings {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            jwt: JwtSettings {
                secret: {
                    let secret = env::var("JWT_SECRET")
                        .map_err(|_| ConfigError::MissingJwtSecret)?;
                    if secret.len() < 32 {
                        return Err(ConfigError::JwtSecretTooShort);
                    }
                    secret
                },
                expires_in: env::var("JWT_EXPIRES_IN")
                    .unwrap_or_else(|_| "3600".to_string()) // 1 hour default
                    .parse()
                    .unwrap_or(3600),
            },
            app: AppSettings {
                environment: env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .parse()
                    .unwrap_or(Environment::Development),
                upload_path: env::var("UPLOAD_PATH")
                    .unwrap_or_else(|_| "./uploads".to_string()),
                max_file_size: env::var("MAX_FILE_SIZE")
                    .unwrap_or_else(|_| "10485760".to_string()) // 10MB default
                    .parse()
                    .unwrap_or(10485760),
            },
            logging: LoggingSettings {
                level: env::var("RUST_LOG")
                    .unwrap_or_else(|_| "info".to_string()),
                file: env::var("LOG_FILE").ok(),
            },
        };

        // Validate configuration
        settings.validate()?;

        Ok(settings)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        // Validate JWT secret length
        if self.jwt.secret.len() < 32 {
            return Err(ConfigError::JwtSecretTooShort);
        }

        // Validate database connection limits
        if self.database.min_connections > self.database.max_connections {
            return Err(ConfigError::InvalidDatabaseConnections);
        }

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        self.app.environment == Environment::Production
    }

    pub fn is_development(&self) -> bool {
        self.app.environment == Environment::Development
    }
}

impl std::str::FromStr for Environment {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(ConfigError::InvalidEnvironment),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("DATABASE_URL environment variable is required")]
    MissingDatabaseUrl,
    
    #[error("JWT_SECRET environment variable is required")]
    MissingJwtSecret,
    
    #[error("JWT_SECRET must be at least 32 characters long for security")]
    JwtSecretTooShort,
    
    #[error("Invalid port number")]
    InvalidPort,
    
    #[error("Invalid environment specified")]
    InvalidEnvironment,
    
    #[error("Database min_connections cannot be greater than max_connections")]
    InvalidDatabaseConnections,
}