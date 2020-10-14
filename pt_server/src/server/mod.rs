use crate::{
    config::Config, logger::LoggerMw, model::error::GenericError,
    services::app_user::DefaultAppUserService, services::AppUserService, services::AuthService,
    services::DefaultAuthService,
};
use actix_cors::Cors;
use actix_web::{web::ServiceConfig, App, HttpServer};
use aide::openapi::v3::{generate_api, transform, ui::ReDoc};
use slog::{info, Logger};

pub mod routes;

pub async fn run(config: Config, logger: Logger, pool: sqlx::PgPool) -> anyhow::Result<()> {
    let host = config.host.clone();
    let port = config.port;

    info!(logger, "server start";
        "host" => &host,
        "port" => port
    );

    // let app_user_service = DefaultAppUserService::new();

    HttpServer::new(move || {
        App::new()
            .wrap(LoggerMw::new(logger.clone()))
            .wrap(Cors::new().allowed_header("All").finish())
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
        let app_user_service = DefaultAppUserService::new(&c, logger.clone(), pool.clone());
        let auth_service = DefaultAuthService::new(&c, logger, pool);

        app.data::<Box<dyn AppUserService>>(Box::new(app_user_service));
        app.data::<Box<dyn AuthService>>(Box::new(auth_service));
    }
}

pub fn configure_routes(config: &Config) -> impl FnOnce(&mut ServiceConfig) {
    let c = config.clone();
    move |app: &mut ServiceConfig| {
        routes::user::configure_routes(&c)(app);
        
        if c.api_docs {
            let api = generate_api(None)
                .unwrap()
                .transform(transform::default_response(
                    "An unexpected error",
                    GenericError {
                        message: "An unexpected error happened".into(),
                    },
                ));

            app.service(
                ReDoc::new()
                    .openapi_v3(&api)
                    .api_at("api.json")
                    .actix_service("/docs"),
            );
        }

    }
}
