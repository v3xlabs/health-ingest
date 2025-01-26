use chrono::{DateTime, Utc};
use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;
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
struct IOSPushBody {
    samples: String,
    name: Option<String>,
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

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct HeartrateSample {
    value: i64,
    start_date: String,
    end_date: i64,
}

impl From<IOSHeartrateSample> for HeartrateSample {
    fn from(sample: IOSHeartrateSample) -> Self {
        Self {
            value: sample.value.parse::<i64>().unwrap(),
            start_date: parse_ios_date(&sample.start_date).to_rfc2822(),
            end_date: parse_ios_date(&sample.end_date).timestamp_nanos() as i64,
        }
    }
}

// input: "24 Jan 2025 at 00:05"
fn parse_ios_date(date: &str) -> DateTime<Utc> {
    info!("date: {:?}", date);

    date.parse::<DateTime<Utc>>().unwrap()
}

#[handler]
async fn push_ios(state: Data<&Arc<AppState>>, body: Json<IOSPushBody>) -> impl IntoResponse {
    let name = body.name.as_deref().unwrap_or_default();

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

    let samples: Vec<DataPoint> = samples
        .into_iter()
        .map(|s| {
            DataPoint::builder("heartrate")
                .field("value", s.value)
                .tag("entity", name)
                .timestamp(s.end_date)
                .build()
                .unwrap()
        })
        .collect();

    client
        .write(&state.config.influxdb.bucket, stream::iter(samples))
        .await
        .unwrap();

    "ok"
}
