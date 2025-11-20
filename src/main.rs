use axum::{extract::State, http::StatusCode, routing::get, Router};
use serde::Deserialize;
use std::fmt::Write;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Attraction {
    Id: String,
    Name: String,
    WaitingTime: Option<i16>,
    Empire: String,
    r#Type: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct EftelingResponse {
    AttractionInfo: Vec<Attraction>,
}

struct CachedResponse {
    data: String,
    timestamp: Instant,
}

struct AppState {
    client: reqwest::Client,
    cache: RwLock<Option<CachedResponse>>,
}

const CACHE_TTL: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "efteling_node_exporter=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create shared state with HTTP client and cache
    let state = Arc::new(AppState {
        client: reqwest::Client::new(),
        cache: RwLock::new(None),
    });

    // build our application with routes supporting both /metrics and /metrics/
    let app = Router::new()
        .route("/metrics", get(fetch_metrics))
        .route("/metrics/", get(fetch_metrics))
        .with_state(state);

    info!("Listening on 0.0.0.0:1337");

    // run our app with hyper, listening globally on port 1337
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1337").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn fetch_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, String), StatusCode> {
    // Check cache first
    {
        let cache = state.cache.read().await;
        if let Some(cached) = cache.as_ref() {
            if cached.timestamp.elapsed() < CACHE_TTL {
                return Ok((StatusCode::OK, cached.data.clone()));
            }
        }
    }

    // Fetch from API using shared client
    let url = "https://api.efteling.com/app/wis";
    let response = state.client.get(url).send().await.map_err(|err| {
        error!("Error fetching metrics: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let json: EftelingResponse = response.json().await.map_err(|err| {
        error!("Error parsing JSON: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let metrics = process_metrics(json)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        *cache = Some(CachedResponse {
            data: metrics.1.clone(),
            timestamp: Instant::now(),
        });
    }

    Ok(metrics)
}

fn escape_label_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

fn process_metrics(json: EftelingResponse) -> Result<(StatusCode, String), StatusCode> {
    let mut response = String::with_capacity(4096);
    response.push_str("# HELP efteling_waiting_time Waiting time for attractions\n# TYPE efteling_waiting_time gauge\n");

    for attraction in json.AttractionInfo {
        if let Some(waiting_time) = attraction.WaitingTime {
            write!(
                response,
                "efteling_waiting_time{{id=\"{}\", name=\"{}\", empire=\"{}\", type=\"{}\"}} {}\n",
                escape_label_value(&attraction.Id),
                escape_label_value(&attraction.Name),
                escape_label_value(&attraction.Empire),
                escape_label_value(&attraction.r#Type),
                waiting_time
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    Ok((StatusCode::OK, response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_metrics() {
        let data = EftelingResponse {
            AttractionInfo: vec![
                Attraction {
                    Id: "1".to_string(),
                    Name: "Attraction 1".to_string(),
                    WaitingTime: Some(10),
                    Empire: "Empire 1".to_string(),
                    r#Type: "Type 1".to_string(),
                },
                Attraction {
                    Id: "2".to_string(),
                    Name: "Attraction 2".to_string(),
                    WaitingTime: None,
                    Empire: "Empire 2".to_string(),
                    r#Type: "Type 2".to_string(),
                },
            ],
        };

        let expected = (
            StatusCode::OK,
            "# HELP efteling_waiting_time Waiting time for attractions\n# TYPE efteling_waiting_time gauge\nefteling_waiting_time{id=\"1\", name=\"Attraction 1\", empire=\"Empire 1\", type=\"Type 1\"} 10\n".to_string(),
        );

        assert_eq!(process_metrics(data).expect("process_metrics should succeed"), expected);
    }
}
