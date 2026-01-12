use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

use reqwest::{header::HeaderMap, Client, ClientBuilder, Response};
use serde_json::{json, Value};

pub struct ApiResponse {
    elapsed: u128,
    response_code: u16,
    body: Value,
}

static CLIENT: OnceLock<Client> = OnceLock::new();

fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Error creating request client! Check logs for more details.")
    })
}

pub async fn get(
    url: &str,
    headers: Option<HeaderMap>,
    params: Option<Value>,
) -> Result<ApiResponse, reqwest::Error> {
    // unwrap arguments
    let headers = match headers {
        Some(v) => v,
        None => HeaderMap::new(),
    };

    let params = params.unwrap_or(json!(""));

    let client = client().clone();
    let start = Instant::now();
    let response = client
        .get(url)
        .headers(headers)
        .json(&params)
        .send()
        .await?;

    Ok(ApiResponse {
        elapsed: start.elapsed().as_millis(),
        response_code: response.status().as_u16(),
        body: response.json().await?,
    })
}

pub async fn post(
    url: &str,
    headers: Option<HeaderMap>,
    params: Option<Value>,
) -> Result<ApiResponse, reqwest::Error> {
    // unwrap arguments
    let headers = match headers {
        Some(v) => v,
        None => HeaderMap::new(),
    };

    let params = params.unwrap_or(json!(""));

    let client = client().clone();
    let start = Instant::now();
    let response = client
        .post(url)
        .headers(headers)
        .json(&params)
        .send()
        .await?;

    Ok(ApiResponse {
        elapsed: start.elapsed().as_millis(),
        response_code: response.status().as_u16(),
        body: response.json().await?,
    })
}

pub async fn get_stream(url: &str, headers: HeaderMap, params: Value) -> Response {
    todo!()
}
