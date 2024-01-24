use log::LevelFilter;
mod aggregator;
mod cli;
#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
    cli::parse_args().await;
}
