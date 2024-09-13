use academy_models::auth::{AuthError, AuthenticateError, AuthorizeError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::models::ApiError;

pub mod config;
pub mod contact;
pub mod health;
pub mod internal;
pub mod mfa;
pub mod oauth2;
pub mod session;
pub mod user;

pub fn internal_server_error(err: impl Into<anyhow::Error>) -> Response {
    let err = err.into();
    tracing::error!("internal server error: {err}");
    error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
}

fn auth_error(err: impl Into<AuthError>) -> Response {
    match err.into() {
        AuthError::Authenticate(AuthenticateError::InvalidToken) => {
            error(StatusCode::UNAUTHORIZED, "Invalid token")
        }
        AuthError::Authenticate(AuthenticateError::Other(err)) => internal_server_error(err),
        AuthError::Authorize(AuthorizeError::Admin) => {
            error(StatusCode::FORBIDDEN, "Permission denied")
        }
        AuthError::Authorize(AuthorizeError::EmailVerified) => {
            error(StatusCode::FORBIDDEN, "Email not verified")
        }
    }
}

fn error(code: StatusCode, detail: &'static str) -> Response {
    (code, Json(ApiError { detail })).into_response()
}
