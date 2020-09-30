use pt_server::{
    config::Config,
    db,
    logger::{create_logger, LoggerExt},
    server,
};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let db_pool = db::connect(&config).await?;
    let log = create_logger(&config).with_scope("http-server");
    server::run(config, log, db_pool).await
}
