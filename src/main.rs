use chrono::{DateTime, Utc};
use futures::stream;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use influxdb2_derive::WriteDataPoint;
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

#[derive(Debug, Deserialize, Serialize)]
pub struct IOSHeartrateSample {
    value: String,
    #[serde(rename = "type")]
    _type: Option<String>,
    #[serde(rename = "unit")]
    unit: Option<String>,
    #[serde(rename = "startDate")]
    start_date: String,
    #[serde(rename = "endDate")]
    end_date: String,
}

#[derive(Debug, Deserialize, Serialize, WriteDataPoint, Default)]
#[measurement = "heartrate"]
pub struct HeartrateSample {
    #[influxdb(field)]
    value: i64,
    #[influxdb(tag)]
    start_date: String,
    #[influxdb(timestamp)]
    end_date: i64,
}

impl From<IOSHeartrateSample> for HeartrateSample {
    fn from(sample: IOSHeartrateSample) -> Self {
        Self {
            value: sample.value.parse::<i64>().unwrap(),
            start_date: parse_ios_date(&sample.start_date).to_rfc2822(),
            end_date: parse_ios_date(&sample.end_date).timestamp(),
        }
    }
}

// input: "24 Jan 2025 at 00:05"
fn parse_ios_date(date: &str) -> DateTime<Utc> {
    info!("date: {:?}", date);

    date.parse::<DateTime<Utc>>().unwrap()
}

#[handler]
async fn push_ios(state: Data<&Arc<AppState>>, body: Json<PushBody>) -> impl IntoResponse {
    let _ = state.config;

    // info!("pushing ios {:?}", body);
    let client = Client::new(
        &state.config.influxdb.url,
        &state.config.influxdb.org,
        &state.config.influxdb.token,
    );

    let samples = body.samples.replace("\\\"", "\"");
    let samples = format!("[{}]", samples);
    let samples: Vec<IOSHeartrateSample> = serde_json::from_str(&samples).unwrap();

    let samples: Vec<HeartrateSample> = samples.into_iter().map(|s| s.into()).collect();

    info!("samples: {:?}", samples);

    client
        .write(&state.config.influxdb.bucket, stream::iter(samples))
        .await
        .unwrap();

    "ok"
}
