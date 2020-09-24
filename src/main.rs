use actix_cors::Cors;
use actix_web::{App, HttpServer};
use pt_server::{
    config::Config,
    logger::{create_logger, LoggerExt, LoggerMw},
    server::routes,
};
use slog::info;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    let log = create_logger(&config).with_scope("http-server");

    let port = config.port.unwrap_or(8080);

    info!(log, "listening";
        "port" => port
    );

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .wrap(LoggerMw::new(log.clone()))
            .wrap(
                Cors::new()
                    .allowed_header("All")
                    .allowed_origin("*")
                    .finish(),
            )
            .configure(routes)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await?;

    Ok(())
}
