use std::fmt::NumBuffer;
use std::sync::Arc;

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const API_URL: &str = "https://api.efteling.com/app/wis";
const LISTEN_ADDR: &str = "0.0.0.0:1337";
const CACHE_TTL: Duration = Duration::from_secs(30);
/// Content type of the Prometheus text exposition format.
const METRICS_CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Attraction {
    id: String,
    name: String,
    waiting_time: Option<i16>,
    empire: String,
    #[serde(rename = "Type")]
    kind: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EftelingResponse {
    attraction_info: Vec<Attraction>,
}

#[derive(Debug)]
struct CachedResponse {
    /// Rendered metrics; `Bytes` makes serving a cache hit a refcount bump instead of a copy.
    body: Bytes,
    timestamp: Instant,
}

#[derive(Debug)]
struct AppState {
    client: reqwest::Client,
    cache: RwLock<Option<CachedResponse>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "efteling_node_exporter=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState {
        client: reqwest::Client::new(),
        cache: RwLock::new(None),
    });

    // Support both /metrics and /metrics/
    let app = Router::new()
        .route("/metrics", get(fetch_metrics))
        .route("/metrics/", get(fetch_metrics))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(LISTEN_ADDR).await?;
    info!("Listening on {LISTEN_ADDR}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn fetch_metrics(State(state): State<Arc<AppState>>) -> Result<Response, StatusCode> {
    if let Some(cached) = state.cache.read().await.as_ref()
        && cached.timestamp.elapsed() < CACHE_TTL
    {
        return Ok(metrics_response(cached.body.clone()));
    }

    let json: EftelingResponse = state
        .client
        .get(API_URL)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|err| {
            error!("Error fetching metrics: {err:?}");
            StatusCode::BAD_GATEWAY
        })?
        .json()
        .await
        .map_err(|err| {
            error!("Error parsing JSON: {err:?}");
            StatusCode::BAD_GATEWAY
        })?;

    let body = Bytes::from(render_metrics(&json));

    *state.cache.write().await = Some(CachedResponse {
        body: body.clone(),
        timestamp: Instant::now(),
    });

    Ok(metrics_response(body))
}

fn metrics_response(body: Bytes) -> Response {
    ([(header::CONTENT_TYPE, METRICS_CONTENT_TYPE)], body).into_response()
}

/// Appends `value` to `out`, escaped as a Prometheus label value.
fn push_label_value(out: &mut String, value: &str) {
    for c in value.chars() {
        match c {
            '\\' => out.push_str(r"\\"),
            '"' => out.push_str(r#"\""#),
            '\n' => out.push_str(r"\n"),
            c => out.push(c),
        }
    }
}

fn render_metrics(json: &EftelingResponse) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str(
        "# HELP efteling_waiting_time Waiting time for attractions\n\
         # TYPE efteling_waiting_time gauge\n",
    );

    let mut num = NumBuffer::new();
    for attraction in &json.attraction_info {
        let Some(waiting_time) = attraction.waiting_time else {
            continue;
        };
        out.push_str("efteling_waiting_time{id=\"");
        push_label_value(&mut out, &attraction.id);
        out.push_str("\", name=\"");
        push_label_value(&mut out, &attraction.name);
        out.push_str("\", empire=\"");
        push_label_value(&mut out, &attraction.empire);
        out.push_str("\", type=\"");
        push_label_value(&mut out, &attraction.kind);
        out.push_str("\"} ");
        out.push_str(waiting_time.format_into(&mut num));
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attraction(id: &str, name: &str, waiting_time: Option<i16>) -> Attraction {
        Attraction {
            id: id.to_owned(),
            name: name.to_owned(),
            waiting_time,
            empire: format!("Empire {id}"),
            kind: format!("Type {id}"),
        }
    }

    #[test]
    fn renders_attractions_with_waiting_time() {
        let data = EftelingResponse {
            attraction_info: vec![
                attraction("1", "Attraction 1", Some(10)),
                attraction("2", "Attraction 2", None),
            ],
        };

        assert_eq!(
            render_metrics(&data),
            "# HELP efteling_waiting_time Waiting time for attractions\n\
             # TYPE efteling_waiting_time gauge\n\
             efteling_waiting_time{id=\"1\", name=\"Attraction 1\", empire=\"Empire 1\", type=\"Type 1\"} 10\n"
        );
    }

    #[test]
    fn escapes_label_values() {
        let data = EftelingResponse {
            attraction_info: vec![attraction("3", "Back\\slash \"quoted\"\nnewline", Some(-1))],
        };

        assert!(render_metrics(&data).ends_with(
            "name=\"Back\\\\slash \\\"quoted\\\"\\nnewline\", empire=\"Empire 3\", type=\"Type 3\"} -1\n"
        ));
    }

    #[test]
    fn deserializes_api_payload() {
        let json = r#"{"AttractionInfo":[{"Id":"baron","Name":"Baron 1898","WaitingTime":25,"Empire":"Ruigrijk","Type":"Attraction","Extra":true}]}"#;
        let parsed: EftelingResponse = serde_json::from_str(json).unwrap();
        let first = &parsed.attraction_info[0];
        assert_eq!(first.id, "baron");
        assert_eq!(first.waiting_time, Some(25));
        assert_eq!(first.kind, "Attraction");
    }
}
