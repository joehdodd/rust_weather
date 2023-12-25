use axum::{
    extract::{Path, Query},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dotenv::dotenv;
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Deserialize)]
struct WeatherQuery {
    location: String,
}

async fn get_weather(location_query: Query<WeatherQuery>) -> Response<axum::body::Body> {
    let location_query = &location_query.location;
    let key = std::env::var("WEATHER_API_KEY").unwrap();
    let request_url = format!(
        "http://api.weatherstack.com/curent?access_key={}&query={}&units=f",
        key, location_query
    );
    let res = reqwest::get(request_url).await;
    match res {
        /*
         * serde_json allows us to 1) give the Value type to arbitraty JSON 2) use the json! macro to create arbitrary JSON
         */
        Ok(response) => {
            let stat = response.headers();
            println!("stat: {:?}", stat);
            let res_json = response.json::<serde_json::Value>().await;
            // in most cases this will always return 200 because if the json is okay, reqwest returns a 200
            match res_json {
                Ok(res) => (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "data": res,
                    })),
                )
                    .into_response(),
                Err(e) => {
                    let error_status = e.status();
                    println!("error_status: {:?}", error_status);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": {
                                "message": "Request to Weather Stack Failed!",
                            },
                        })),
                    )
                        .into_response()
                }
            }
        }
        Err(_e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "message": "Internal server error.",
                },
            })),
        )
            .into_response(),
    }
}

async fn get_person(Path(person_id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    let request_url = format!("https://swapi.dev/api/people/{}", person_id);
    let response = reqwest::get(request_url).await?;
    Ok(Json(serde_json::json!({
        "data": response.json::<serde_json::Value>().await?,
        "success": true
    })))
    // if response.status().is_success() {
    //     let res_json = response.json::<serde_json::Value>().await?;
    //     Ok(Json(serde_json::json!({
    //         "data": res_json,
    //         "success": true
    //     })))
    // } else {
    //     let res_string = response.text().await?;
    //     Err(AppError(anyhow::anyhow!("{}", res_string)))
    // }
}

#[derive(Debug)]
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        println!("Error: {:?}", self);
        let body = Json(serde_json::json!({
            "error": self.0.to_string(),
        }));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
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
        .route("/", get(get_weather))
        .route("/person/:person_id", get(get_person))
        .layer(cors)
        .with_state(client);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1337")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
