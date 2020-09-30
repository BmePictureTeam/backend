use postgres::Client;

mod migrations {
    use refinery::embed_migrations;
    embed_migrations!("../migrations");
}

pub fn migrate(conn_url: &str) -> anyhow::Result<()> {
    let mut client = Client::connect(conn_url, postgres::NoTls)?;
    let report: refinery::Report = migrations::migrations::runner().run(&mut client)?;

    for m in report.applied_migrations() {
        println!("applied migration: #{} {}", m.version(), m.name());
    }

    println!("The database is up to date.");

    Ok(())
}
