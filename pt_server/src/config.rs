use serde::{Deserialize, Serialize};

pub static CONFIG_ENV_PREFIX: &str = "PT_";

/// Available configuration values.
/// These are mapped to SCREAMING_UPPER_CASE environment variables.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// Server host
    pub host: String,

    /// Server port
    pub port: usize,

    /// Log output as JSON.
    pub log_json: bool,

    /// Postgres database URL.
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let c = envy::prefixed(CONFIG_ENV_PREFIX).from_env()?;
        Ok(c)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 8080,
            log_json: false,
            database_url: "postgresql://postgres:postgres@localhost:5432/postgres".into(),
        }
    }
}
