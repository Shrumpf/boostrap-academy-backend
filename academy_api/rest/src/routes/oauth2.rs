use std::sync::Arc;

use academy_core_oauth2_contracts::OAuth2Service;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};

use crate::models::oauth2::ApiOAuth2ProviderSummary;

pub fn router(service: Arc<impl OAuth2Service>) -> Router<()> {
    Router::new()
        .route("/auth/oauth/providers", routing::get(list_providers))
        .route(
            "/auth/oauth/links/:user_id",
            routing::get(list_links).post(create_link),
        )
        .route(
            "/auth/oauth/links/:user_id/:link_id",
            routing::delete(delete_link),
        )
        .with_state(service)
}

async fn list_providers(service: State<Arc<impl OAuth2Service>>) -> Response {
    Json(
        service
            .list_providers()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ApiOAuth2ProviderSummary>>(),
    )
    .into_response()
}

async fn list_links() {}

async fn create_link() {}

async fn delete_link() {}
