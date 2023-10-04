#![allow(clippy::needless_return)]

use axum::{
    extract,
    http::{header::HeaderMap, StatusCode},
    response,
    routing::{get, post},
    Router,
};
use axum_macros::debug_handler;
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};

async fn check_auth(auth: &str) -> Result<(), WebhookError> {
    let first_7 = &auth[0..8];
    if first_7 != "Bearer " {
        return Ok(());
    } else {
        return Err(WebhookError::Unauthorized);
    }
}

#[debug_handler]
async fn echo_app(
    extract::Path(app_name): extract::Path<String>,
    headers: HeaderMap,
    extract::Json(body): extract::Json<Value>,
) -> Result<response::Json<Value>, WebhookError> {
    let auth = match headers.get("Authorization") {
        Some(auth) => auth.to_str().unwrap(),
        None => return Err(WebhookError::NoAuthHeader),
    };
    check_auth(auth).await?;
    let resp_msg = format!("Running {:?}, Auth {:?}", app_name, auth);

    return Ok(response::Json(json!({
        "status": "OK",
        "result": resp_msg,
        "body": body,
    })));
}

async fn health() -> response::Json<Value> {
    return response::Json(json!({"status": "OK"}));
}

#[derive(Debug)]
enum WebhookError {
    NoAuthHeader,
    Unauthorized,
}
impl response::IntoResponse for WebhookError {
    fn into_response(self) -> response::Response {
        let (status_code, message) = match self {
            WebhookError::NoAuthHeader => {
                (StatusCode::BAD_REQUEST, "No Authorization header found.")
            }
            WebhookError::Unauthorized => (StatusCode::UNAUTHORIZED, "User is not authorized."),
        };

        let body = response::Json(json!({
            "status": "Error",
            "detail": message,
        }));

        return (status_code, body).into_response();
    }
}

#[tokio::main]
async fn main() -> Result<(), WebhookError> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/:app_name", post(echo_app))
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_credentials(false),
        );
    axum::Server::bind(&"0.0.0.0:80".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    return Ok(());
}
