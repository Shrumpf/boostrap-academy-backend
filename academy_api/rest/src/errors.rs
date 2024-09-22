use academy_models::auth::{AuthError, AuthenticateError, AuthorizeError};
use aide::transform::TransformOperation;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{const_schema, docs::TransformOperationExt};

pub fn internal_server_error(err: impl Into<anyhow::Error>) -> Response {
    let err = err.into();
    tracing::error!("internal server error: {err}");
    error(StatusCode::INTERNAL_SERVER_ERROR, InternalServerErrorDetail)
}

pub fn internal_server_error_docs(op: TransformOperation) -> TransformOperation {
    op.add_response::<ApiError<InternalServerErrorDetail>>(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error",
    )
}

pub fn auth_error(err: impl Into<AuthError>) -> Response {
    match err.into() {
        AuthError::Authenticate(AuthenticateError::InvalidToken) => {
            error(StatusCode::UNAUTHORIZED, InvalidTokenDetail)
        }
        AuthError::Authenticate(AuthenticateError::Other(err)) => internal_server_error(err),
        AuthError::Authorize(AuthorizeError::Admin) => {
            error(StatusCode::FORBIDDEN, PermissionDeniedDetail).into_response()
        }
        AuthError::Authorize(AuthorizeError::EmailVerified) => {
            error(StatusCode::FORBIDDEN, EmailNotVerifiedDetail).into_response()
        }
    }
}

pub fn auth_error_docs(op: TransformOperation) -> TransformOperation {
    op.add_response::<ApiError<InvalidTokenDetail>>(
        StatusCode::UNAUTHORIZED,
        "The authentication token is invalid or has expired.",
    )
    .with(internal_server_error_docs)
    .add_response::<ApiError<PermissionDeniedDetail>>(
        StatusCode::FORBIDDEN,
        "The authenticated user is not allowed to perform this action.",
    )
    .add_response::<ApiError<EmailNotVerifiedDetail>>(
        StatusCode::FORBIDDEN,
        "The authenticated user has not verified their email address.",
    )
}

pub fn recaptcha_error_docs(op: TransformOperation) -> TransformOperation {
    op.add_response::<ApiError<RecaptchaFailedDetail>>(
        StatusCode::PRECONDITION_FAILED,
        "reCAPTCHA is enabled but no valid reCAPTCHA response has been provided.",
    )
}

pub fn error(code: StatusCode, detail: impl Serialize) -> Response {
    (code, Json(ApiError { detail })).into_response()
}

#[derive(Serialize, JsonSchema, Default)]
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
    pub UserAlreadyExistsDetail("User already exists");
    pub EmailAlreadyExistsDetail("Email already exists");
    pub NoLoginMethodDetail("No login method");
    pub InvalidOAuthTokenDetail("Invalid OAuth token");
    pub RemoteAlreadyLinkedDetail("Remote already linked");
}
