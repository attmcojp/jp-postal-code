use anyhow::Context;
use clap::Parser;
use jp_postal_code::{config, infra, usecase, MIGRATOR};
use tracing_subscriber::prelude::*;

#[derive(Parser)]
#[command(name = "jp-postal-code-update-database")]
#[command(about = "Update Japanese postal code database")]
struct Cli {
    #[arg(long, help = "Custom URL for utf_ken_all.zip (optional)")]
    url: Option<String>,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jp_postal_code_update_database=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let Err(err) = run().await {
        tracing::error!(?err, "Failed to update postal code database");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let conf = config::Config::new();

    tracing::info!("Connecting to database...");
    let pool = sqlx::PgPool::connect(conf.database_url.as_ref())
        .await
        .context("Failed to connect to database")?;

    tracing::info!("Running database migrations...");
    MIGRATOR
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    let mut repo = infra::postgres::UtfKenAllRepositoryPostgres::new(pool);

    tracing::info!("Starting postal code database update...");
    usecase::update_postal_code_database(&mut repo, cli.url)
        .await
        .context("Failed to update postal code database")?;

    tracing::info!("Postal code database updated successfully!");
    Ok(())
}
