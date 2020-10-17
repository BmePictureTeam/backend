use crate::{
    config::Config, model::error::GenericError, services::AuthService,
    services::DefaultAuthService, services::DefaultImageService, services::ImageService,
};
use actix_cors::Cors;
use actix_web::{web::ServiceConfig, App, HttpServer};
use aide::openapi::v3::{generate_api, transform, ui::ReDoc};
use slog::{info, Logger};

pub mod extractors;
pub mod middleware;
pub mod routes;

pub async fn run(config: Config, logger: Logger, pool: sqlx::PgPool) -> anyhow::Result<()> {
    let host = config.host.clone();
    let port = config.port;

    info!(logger, "server start";
        "host" => &host,
        "port" => port
    );

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new(logger.clone()))
            .wrap(Cors::new().allowed_header("All").finish())
            .data(logger.clone())
            .configure(configure_services(&config, logger.clone(), pool.clone()))
            .configure(configure_routes(&config))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;

    Ok(())
}

pub fn configure_services(
    config: &Config,
    logger: Logger,
    pool: sqlx::PgPool,
) -> impl FnOnce(&mut ServiceConfig) {
    let c = config.clone();
    move |app: &mut ServiceConfig| {
        let auth_service = DefaultAuthService::new(&c, logger.clone(), pool.clone());
        let image_service = DefaultImageService::new(&c, logger, pool);

        app.data::<Box<dyn AuthService>>(Box::new(auth_service));
        app.data::<Box<dyn ImageService>>(Box::new(image_service));
    }
}

pub fn configure_routes(config: &Config) -> impl FnOnce(&mut ServiceConfig) {
    let c = config.clone();
    move |app: &mut ServiceConfig| {
        routes::auth::configure_routes(&c)(app);
        routes::image::configure_routes(&c)(app);
        routes::category::configure_routes(&c)(app);

        if c.api_docs {
            let api = generate_api(None)
                .unwrap()
                .transform(transform::default_response(
                    "An unexpected error",
                    GenericError {
                        message: "An unexpected error happened".into(),
                    },
                ))
                .transform(|mut api| {
                    api.tags.sort_by(|a, b| a.name.cmp(&b.name));
                    api
                });

            app.service(
                ReDoc::new()
                    .openapi_v3(&api)
                    .api_at("api.json")
                    .actix_service("/docs"),
            );
        }
    }
}
