use crate::config::Config;
use sqlx::postgres::PgPoolOptions;

pub async fn connect(config: &Config) -> anyhow::Result<sqlx::PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?)
}
