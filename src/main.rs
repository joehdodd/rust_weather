use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    #[serde(rename = "type")]
    weather_type: String,
    query: String,
    language: String,
    unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Location {
    name: String,
    country: String,
    region: String,
    lat: String,
    lon: String,
    timezone_id: String,
    localtime: String,
    localtime_epoch: i64,
    utc_offset: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Current {
    observation_time: String,
    temperature: i64,
    weather_code: i64,
    weather_icons: Vec<String>,
    weather_descriptions: Vec<String>,
    wind_speed: i64,
    wind_degree: i64,
    wind_dir: String,
    pressure: i64,
    precip: i64,
    humidity: i64,
    cloudcover: i64,
    feelslike: i64,
    uv_index: i64,
    visibility: i64,
    is_day: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeatherData {
    request: Request,
    location: Location,
    current: Current,
}

async fn get_todos() -> Json<WeatherData> {
    let key = std::env::var("WEATHER_API_KEY").unwrap();
    let request_url = format!("http://api.weatherstack.com/current?access_key={}&query=New York", key);
    let client = reqwest::Client::new();
    let res = client.get(&request_url)
        .send()
        .await
        .unwrap();
    let data: WeatherData = res.json().await.unwrap();
    Json(data)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let app = Router::new().route("/", get(get_todos));
    let listener = tokio::net::TcpListener::bind("localhost:1337")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
