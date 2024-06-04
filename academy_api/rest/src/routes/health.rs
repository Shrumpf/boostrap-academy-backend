use std::sync::Arc;

use academy_core_health_contracts::{HealthService, HealthStatus};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use serde::Serialize;

pub fn router(service: Arc<impl HealthService>) -> Router<()> {
    Router::new()
        .route("/health", routing::get(health))
        .with_state(service)
}

#[derive(Serialize)]
struct HealthResponse {
    http: bool,
    database: bool,
    cache: bool,
    email: bool,
}

async fn health(service: State<Arc<impl HealthService>>) -> Response {
    let HealthStatus {
        database,
        cache,
        email,
    } = service.get_status().await;

    let ok = database && cache && email;

    let status = if ok {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    let response = HealthResponse {
        http: true,
        database,
        cache,
        email,
    };

    (status, Json(response)).into_response()
}
