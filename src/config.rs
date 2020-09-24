use serde::{Deserialize, Serialize};

pub static CONFIG_ENV_PREFIX: &str = "PT_";

/// Available configuration values.
/// These are mapped to SCREAMING_UPPER_CASE environment variables.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<usize>,
    #[serde(default)]
    /// Log output as JSON.
    pub log_json: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        envy::prefixed(CONFIG_ENV_PREFIX).from_env().map_err(Into::into)
    }
}
