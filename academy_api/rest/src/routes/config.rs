use std::sync::Arc;

use academy_core_config_contracts::ConfigFeatureService;
use aide::{
    axum::{routing, ApiRouter},
    transform::TransformOperation,
};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

pub const TAG: &str = "Config";

pub fn router(service: Arc<impl ConfigFeatureService>) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route(
            "/auth/recaptcha",
            routing::get_with(get_recaptcha_sitekey, get_recaptcha_sitekey_docs),
        )
        .with_state(service)
}

async fn get_recaptcha_sitekey(service: State<Arc<impl ConfigFeatureService>>) -> Response {
    Json(service.get_recaptcha_sitekey()).into_response()
}

fn get_recaptcha_sitekey_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Return the public reCAPTCHA sitekey.")
        .description("Returns `null` if reCAPTCHA is disabled.")
        .response_with::<200, Json<Option<&str>>, _>(|op| op.example("recaptcha-sitekey"))
        .tag(TAG)
}
