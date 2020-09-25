use clap::App;
use pt_server::{
    config::Config,
    db,
    logger::{create_logger, LoggerExt},
    server,
};

fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    let matches = App::new("Picture Team Server")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Server for the Picture Team applications")
        .subcommand(App::new("migrate").about("Runs the database migrations"))
        .get_matches();

    let log = create_logger(&config).with_scope("http-server");

    match matches.subcommand_name() {
        Some("migrate") => Ok(db::migrate(&config, log)?),
        None => actix_web::rt::System::new("http-server").block_on(async move {
            let db_pool = db::connect(&config).await?;
            Ok(server::run(config, log, db_pool).await?)
        }),
        _ => unreachable!(),
    }
}
