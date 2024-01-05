use axum::{http::StatusCode, routing::get, Router};
use serde::Deserialize;

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

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/metrics", get(fetch_metrics));

    println!("Listening on 1337");

    // run our app with hyper, listening globally on port 1337
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1337").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn fetch_metrics() -> Result<(StatusCode, String), StatusCode> {
    let url = "https://api.efteling.com/app/wis";
    let response = reqwest::get(url).await.map_err(|err| {
        eprintln!("Error fetching metrics: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let json: EftelingResponse = response.json().await.map_err(|err| {
        eprintln!("Error parsing JSON: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut response = format!(
        "# HELP efteling_waiting_time Waiting time for attractions\n# TYPE efteling_waiting_time gauge\n",
    );

    for attraction in json.AttractionInfo {
        if let Some(waiting_time) = attraction.WaitingTime {
            response.push_str(&format!(
                "efteling_waiting_time{{id=\"{}\", name=\"{}\", empire=\"{}\", type=\"{}\"}} {}\n",
                attraction.Id, attraction.Name, attraction.Empire, attraction.r#Type, waiting_time
            ));
        }
    }

    Ok((StatusCode::OK, response))
}
