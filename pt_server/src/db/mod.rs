use crate::config::Config;
use sqlx::postgres::PgPoolOptions;

pub mod app_user;
pub mod image;
pub mod category;
pub mod rating;

pub async fn connect(config: &Config) -> anyhow::Result<sqlx::PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?)
}
