use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dotenv::dotenv;
use reqwest::Client;
use tower_http::cors::{Any, CorsLayer};

async fn proxy_via_reqwest(State(client): State<Client>) -> Response {
    let request_url = format!("https://swapi.dev/api/people/{}", "1".to_owned());
    let reqwest_response = match client.get(request_url).send().await {
        Ok(res) => res,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Body::empty()).into_response();
        }
    };

    let response_builder = Response::builder().status(reqwest_response.status().as_u16());
    let mut headers = HeaderMap::with_capacity(reqwest_response.headers().len());
    headers.extend(reqwest_response.headers().into_iter().map(|(name, value)| {
        let name = HeaderName::from_bytes(name.as_ref()).unwrap();
        let value = HeaderValue::from_bytes(value.as_ref()).unwrap();
        (name, value)
    }));

    let res_to_client = response_builder
        .body(Body::from_stream(reqwest_response.bytes_stream()));
    match res_to_client {
        Ok(res) => res,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Body::empty()).into_response();
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = reqwest::Client::new();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any);

    let app = Router::new()
        .route("/person", get(proxy_via_reqwest))
        .layer(cors)
        .with_state(client);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1337")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

