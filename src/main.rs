use async_std::prelude::FutureExt;
use state::AppState;
use std::sync::Arc;
use tracing::info;

pub mod state;

#[async_std::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    info!("Starting health-ingest");

    let state = Arc::new(AppState::new().await);

    //
}
