use std::sync::Arc;

use academy_core_oauth2_contracts::{
    OAuth2CreateLinkError, OAuth2CreateSessionError, OAuth2CreateSessionResponse,
    OAuth2DeleteLinkError, OAuth2FeatureService, OAuth2ListLinksError,
};
use academy_models::{
    oauth2::{OAuth2LinkId, OAuth2RegistrationToken},
    session::DeviceName,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use serde::Serialize;

use super::{auth_error, error, internal_server_error};
use crate::{
    extractors::{auth::ApiToken, user_agent::UserAgent},
    models::{
        oauth2::{ApiOAuth2Link, ApiOAuth2Login, ApiOAuth2ProviderSummary},
        session::ApiLogin,
        user::ApiUserIdOrSelf,
    },
};

pub fn router(service: Arc<impl OAuth2FeatureService>) -> Router<()> {
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
        .route("/auth/sessions/oauth", routing::post(create_session))
        .with_state(service)
}

async fn list_providers(service: State<Arc<impl OAuth2FeatureService>>) -> Response {
    Json(
        service
            .list_providers()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ApiOAuth2ProviderSummary>>(),
    )
    .into_response()
}

async fn list_links(
    service: State<Arc<impl OAuth2FeatureService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
) -> Response {
    match service.list_links(&token.0, user_id.into()).await {
        Ok(links) => Json(
            links
                .into_iter()
                .map(Into::into)
                .collect::<Vec<ApiOAuth2Link>>(),
        )
        .into_response(),
        Err(OAuth2ListLinksError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(OAuth2ListLinksError::Auth(err)) => auth_error(err),
        Err(OAuth2ListLinksError::Other(err)) => internal_server_error(err),
    }
}

async fn create_link(
    service: State<Arc<impl OAuth2FeatureService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
    Json(login): Json<ApiOAuth2Login>,
) -> Response {
    match service
        .create_link(&token.0, user_id.into(), login.into())
        .await
    {
        Ok(link) => Json(ApiOAuth2Link::from(link)).into_response(),
        Err(OAuth2CreateLinkError::InvalidProvider) => {
            error(StatusCode::NOT_FOUND, "Provider not found")
        }
        Err(OAuth2CreateLinkError::InvalidCode) => error(StatusCode::UNAUTHORIZED, "Invalid code"),
        Err(OAuth2CreateLinkError::RemoteAlreadyLinked) => {
            error(StatusCode::CONFLICT, "Remote already linked")
        }
        Err(OAuth2CreateLinkError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(OAuth2CreateLinkError::Auth(err)) => auth_error(err),
        Err(OAuth2CreateLinkError::Other(err)) => internal_server_error(err),
    }
}

async fn delete_link(
    service: State<Arc<impl OAuth2FeatureService>>,
    token: ApiToken,
    Path((user_id, link_id)): Path<(ApiUserIdOrSelf, OAuth2LinkId)>,
) -> Response {
    match service.delete_link(&token.0, user_id.into(), link_id).await {
        Ok(()) => Json(true).into_response(),
        Err(OAuth2DeleteLinkError::NotFound) => {
            error(StatusCode::NOT_FOUND, "Connection not found")
        }
        Err(OAuth2DeleteLinkError::CannotRemoveLink) => {
            error(StatusCode::FORBIDDEN, "Cannot delete last login method")
        }
        Err(OAuth2DeleteLinkError::Auth(err)) => auth_error(err),
        Err(OAuth2DeleteLinkError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Serialize)]
enum CreateSessionResponse {
    #[serde(rename = "login")]
    Login(Box<ApiLogin>),
    #[serde(rename = "register_token")]
    RegisterToken(OAuth2RegistrationToken),
}

async fn create_session(
    service: State<Arc<impl OAuth2FeatureService>>,
    user_agent: UserAgent,
    Json(login): Json<ApiOAuth2Login>,
) -> Response {
    match service
        .create_session(
            login.into(),
            user_agent.0.map(DeviceName::from_string_truncated),
        )
        .await
    {
        Ok(OAuth2CreateSessionResponse::Login(login)) => {
            Json(CreateSessionResponse::Login(ApiLogin::from(*login).into())).into_response()
        }
        Ok(OAuth2CreateSessionResponse::RegistrationToken(register_token)) => {
            Json(CreateSessionResponse::RegisterToken(register_token)).into_response()
        }
        Err(OAuth2CreateSessionError::InvalidProvider) => {
            error(StatusCode::NOT_FOUND, "Provider not found")
        }
        Err(OAuth2CreateSessionError::InvalidCode) => {
            error(StatusCode::UNAUTHORIZED, "Invalid code")
        }
        Err(OAuth2CreateSessionError::UserDisabled) => {
            error(StatusCode::FORBIDDEN, "User disabled")
        }
        Err(OAuth2CreateSessionError::Other(err)) => internal_server_error(err),
    }
}
