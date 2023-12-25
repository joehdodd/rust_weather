use axum::{
    error_handling::HandleError,
    extract::{Query, Request},
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

async fn get_person(/*
    Is it possible to extract the person_id here given my implemntation?
     */) -> Result<Json<serde_json::Value>, reqwest::Error> {
    let request_url = format!("https://swapi.dev/api/people/{}", "1asdf".to_owned());
    let response = reqwest::get(request_url).await?;
    if response.status().is_success() {
        let json = response.json::<serde_json::Value>().await?;
        Ok(Json(json))
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = reqwest::Client::new();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any);

    let faillible_person_service = tower::service_fn(|req: Request| async {
        let body = get_person().await?;
        Ok::<_, reqwest::Error>(body.into_response())
    });

    let app = Router::new()
        .route("/", get(get_weather))
        .route_service(
            "/person/:person_id",
            HandleError::new(faillible_person_service, handle_reqwest_error),
        )
        .layer(cors)
        .with_state(client);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1337")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_reqwest_error(err: reqwest::Error) -> Response {
    /*
    
     */
    let status = err.status();
    let status = status.unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR);
    let status_as_u16 = status.as_u16();
    let axum_status =
        StatusCode::from_u16(status_as_u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let res_json = Json(serde_json::json!({
        "error": {
            "message": format!("Something went wrong: {}", err),
        },
    }));
    return (axum_status, res_json).into_response();
}
