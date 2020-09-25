use crate::config::Config;
use postgres::Client;
use slog::{debug, info, Logger};
use sqlx::postgres::PgPoolOptions;

mod migrations;

pub fn migrate(config: &Config, log: Logger) -> anyhow::Result<()> {
    let mut client = Client::connect(&config.database_url, postgres::NoTls)?;
    let report: refinery::Report = migrations::migrations::runner().run(&mut client)?;

    info!(log, "database migrations applied");

    for m in report.applied_migrations() {
        debug!(log, "migration";
            "name" => m.name(),
            "version" => m.version(),
            "applied_on" => m.applied_on().map(|d| d.to_rfc3339()),
            "checksum" => m.checksum()
        );
    }

    Ok(())
}

pub async fn connect(config: &Config) -> anyhow::Result<sqlx::PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?)
}
