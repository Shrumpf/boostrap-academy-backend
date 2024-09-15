use std::sync::Arc;

use academy_auth_contracts::internal::AuthInternalAuthenticateError;
use academy_core_internal_contracts::{
    InternalGetUserByEmailError, InternalGetUserError, InternalService,
};
use academy_models::{email_address::EmailAddress, user::UserId};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};

use super::{error, internal_server_error};
use crate::{extractors::auth::ApiToken, models::user::ApiUser};

pub fn router(service: Arc<impl InternalService>) -> Router<()> {
    Router::new()
        .route("/auth/_internal/users/:user_id", routing::get(get_user))
        .route(
            "/auth/_internal/users/by_email/:email",
            routing::get(get_user_by_email),
        )
        .with_state(service)
}

async fn get_user(
    service: State<Arc<impl InternalService>>,
    token: ApiToken,
    Path(user_id): Path<UserId>,
) -> Response {
    match service.get_user(&token.0, user_id).await {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(InternalGetUserError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(InternalGetUserError::Auth(AuthInternalAuthenticateError::InvalidToken)) => {
            error(StatusCode::UNAUTHORIZED, "Invalid token")
        }
        Err(InternalGetUserError::Other(err)) => internal_server_error(err),
    }
}

async fn get_user_by_email(
    service: State<Arc<impl InternalService>>,
    token: ApiToken,
    Path(email): Path<EmailAddress>,
) -> Response {
    match service.get_user_by_email(&token.0, email).await {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(InternalGetUserByEmailError::NotFound) => {
            error(StatusCode::NOT_FOUND, "User not found")
        }
        Err(InternalGetUserByEmailError::Auth(AuthInternalAuthenticateError::InvalidToken)) => {
            error(StatusCode::UNAUTHORIZED, "Invalid token")
        }
        Err(InternalGetUserByEmailError::Other(err)) => internal_server_error(err),
    }
}
