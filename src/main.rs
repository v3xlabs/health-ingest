use poem::{
    handler,
    listener::TcpListener,
    post,
    web::{Data, Json},
    EndpointExt, IntoResponse, Route, Server,
};
use serde::{Deserialize, Serialize};
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

    let app = Route::new().at("/push/ios", post(push_ios)).data(state);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
        .unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
struct PushBody {
    samples: String,
}

#[handler]
async fn push_ios(state: Data<&Arc<AppState>>, body: Json<PushBody>) -> impl IntoResponse {
    let _ = state.config;

    info!("pushing ios {:?}", body);

    "ok"
}
