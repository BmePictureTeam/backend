use actix_cors::Cors;
use actix_web::{App, HttpServer};
use slog::{info, Logger};

use crate::{config::Config, logger::LoggerMw};

pub mod routes;

pub async fn run(config: Config, log: Logger, db: sqlx::PgPool) -> anyhow::Result<()> {
    let host = config.host.clone();
    let port = config.port;

    info!(log, "server start";
        "host" => &host,
        "port" => port
    );

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .data(db.clone())
            .wrap(LoggerMw::new(log.clone()))
            .wrap(
                Cors::new()
                    .allowed_header("All")
                    .allowed_origin("*")
                    .finish(),
            )
            .configure(routes::setup_routes)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;

    Ok(())
}
