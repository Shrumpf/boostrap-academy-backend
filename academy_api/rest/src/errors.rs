use academy_models::auth::{AuthError, AuthenticateError, AuthorizeError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::const_schema;

pub fn internal_server_error(err: impl Into<anyhow::Error>) -> Response {
    let err = err.into();
    tracing::error!("internal server error: {err}");
    error(StatusCode::INTERNAL_SERVER_ERROR, InternalServerErrorDetail)
}

pub fn auth_error(err: impl Into<AuthError>) -> Response {
    match err.into() {
        AuthError::Authenticate(AuthenticateError::InvalidToken) => {
            error(StatusCode::UNAUTHORIZED, InvalidTokenDetail)
        }
        AuthError::Authenticate(AuthenticateError::Other(err)) => internal_server_error(err),
        AuthError::Authorize(AuthorizeError::Admin) => {
            error(StatusCode::FORBIDDEN, PermissionDeniedDetail)
        }
        AuthError::Authorize(AuthorizeError::EmailVerified) => {
            error(StatusCode::FORBIDDEN, EmailNotVerifiedDetail)
        }
    }
}

pub fn error(code: StatusCode, detail: impl Serialize) -> Response {
    (code, Json(ApiError { detail })).into_response()
}

#[derive(Serialize, JsonSchema)]
pub struct ApiError<D> {
    pub detail: D,
}

const_schema! {
    pub InternalServerErrorDetail("Internal server error");
    pub RecaptchaFailedDetail("Recaptcha failed");

    // Auth
    pub InvalidTokenDetail("Invalid token");
    pub PermissionDeniedDetail("Permission denied");
    pub EmailNotVerifiedDetail("Email not verified");

    // Contact
    pub CoundNotSendMessageDetail("Could not send message");

    // User
    pub UserNotFoundDetail("User not found");
}
