use actix_cors::Cors;
use actix_web::{App, HttpServer};
use aide::openapi::v3::{generate_api, transform, ui::ReDoc};
use slog::{info, Logger};

use crate::{config::Config, logger::LoggerMw, model::error::GenericError};

pub mod routes;

pub async fn run(config: Config, log: Logger, db: sqlx::PgPool) -> anyhow::Result<()> {
    let host = config.host.clone();
    let port = config.port;

    info!(log, "server start";
        "host" => &host,
        "port" => port
    );

    let api = generate_api(None)?.transform(transform::default_response(
        "An unexpected error",
        GenericError {
            message: "An unexpected error happened".into(),
        },
    ));

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .data(db.clone())
            .wrap(LoggerMw::new(log.clone()))
            .wrap(
                Cors::new()
                    .allowed_header("All")
                    .finish(),
            )
            .configure(routes::setup_routes)
            .service(
                ReDoc::new()
                    .openapi_v3(&api)
                    .api_at("api.json")
                    .actix_service("/"),
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;

    Ok(())
}
