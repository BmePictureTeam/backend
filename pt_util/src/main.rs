use std::env;

use clap::{App, AppSettings};

mod db;

fn main() -> anyhow::Result<()> {
    let matches = App::new("Picture Team Utility")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Utility for The Picture Team applications")
        .subcommand(App::new("migrate").about("Runs the database migrations"))
        .get_matches();

    match matches.subcommand_name() {
        Some("migrate") => Ok(db::migrate(
            &env::var("PT_DATABASE_URL").expect("PT_DATABASE_URL env var is required"),
        )?),
        _ => unreachable!(),
    }
}
