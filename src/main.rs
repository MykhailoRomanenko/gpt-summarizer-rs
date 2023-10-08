use clap::Parser;
use summarize::config::{AppConfig, Args};
use summarize::run;

#[tokio::main]
async fn main() -> Result<(), summarize::error::AppError> {
    dotenvy::dotenv().ok();

    let subscriber = tracing_subscriber::FmtSubscriber::default();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let args = Args::parse();
    run(AppConfig::new(args).unwrap()).await
}
