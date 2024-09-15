use std::sync::Arc;

use academy_core_config_contracts::ConfigFeatureService;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};

pub fn router(service: Arc<impl ConfigFeatureService>) -> Router<()> {
    Router::new()
        .route("/auth/recaptcha", routing::get(get_recaptcha_sitekey))
        .with_state(service)
}

async fn get_recaptcha_sitekey(service: State<Arc<impl ConfigFeatureService>>) -> Response {
    Json(service.get_recaptcha_sitekey()).into_response()
}
