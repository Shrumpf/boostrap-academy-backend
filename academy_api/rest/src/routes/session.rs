use std::sync::Arc;

use academy_core_session_contracts::{
    SessionCreateCommand, SessionCreateError, SessionDeleteByUserError, SessionDeleteCurrentError,
    SessionDeleteError, SessionGetCurrentError, SessionImpersonateError, SessionListByUserError,
    SessionRefreshError, SessionService,
};
use academy_models::{
    mfa::{MfaAuthenticateCommand, MfaRecoveryCode, TotpCode},
    session::{DeviceName, SessionId},
    user::{UserId, UserNameOrEmailAddress, UserPassword},
    RecaptchaResponse,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use serde::Deserialize;

use super::{auth_error, error, internal_server_error};
use crate::{
    extractors::{auth::ApiToken, user_agent::UserAgent},
    models::{
        session::{ApiLogin, ApiSession},
        user::ApiUserIdOrSelf,
    },
};

pub fn router(service: Arc<impl SessionService>) -> Router<()> {
    Router::new()
        .route(
            "/auth/session",
            routing::get(get_current)
                .put(refresh)
                .delete(delete_current),
        )
        .route("/auth/sessions", routing::post(create))
        .route(
            "/auth/sessions/:user_id",
            routing::get(list_by_user)
                .post(impersonate)
                .delete(delete_by_user),
        )
        .route(
            "/auth/sessions/:user_id/:session_id",
            routing::delete(delete),
        )
        .with_state(service)
}

async fn get_current(
    session_service: State<Arc<impl SessionService>>,
    token: ApiToken,
) -> Response {
    match session_service.get_current_session(&token.0).await {
        Ok(session) => Json(ApiSession::from(session)).into_response(),
        Err(SessionGetCurrentError::Auth(err)) => auth_error(err),
        Err(SessionGetCurrentError::Other(err)) => internal_server_error(err),
    }
}

async fn list_by_user(
    session_service: State<Arc<impl SessionService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
) -> Response {
    match session_service.list_by_user(&token.0, user_id.into()).await {
        Ok(sessions) => Json(
            sessions
                .into_iter()
                .map(Into::into)
                .collect::<Vec<ApiSession>>(),
        )
        .into_response(),
        Err(SessionListByUserError::Auth(err)) => auth_error(err),
        Err(SessionListByUserError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct CreateRequest {
    name_or_email: UserNameOrEmailAddress,
    password: UserPassword,
    mfa_code: Option<TotpCode>,
    recovery_code: Option<MfaRecoveryCode>,
    recaptcha_response: Option<RecaptchaResponse>,
}

async fn create(
    session_service: State<Arc<impl SessionService>>,
    user_agent: UserAgent,
    Json(CreateRequest {
        name_or_email,
        password,
        mfa_code,
        recovery_code,
        recaptcha_response,
    }): Json<CreateRequest>,
) -> Response {
    match session_service
        .create_session(
            SessionCreateCommand {
                name_or_email,
                password,
                device_name: user_agent.0.map(DeviceName::from_string_truncated),
                mfa: MfaAuthenticateCommand {
                    totp_code: mfa_code,
                    recovery_code,
                },
            },
            recaptcha_response,
        )
        .await
    {
        Ok(result) => (StatusCode::CREATED, Json(ApiLogin::from(result))).into_response(),
        Err(SessionCreateError::InvalidCredentials) => {
            error(StatusCode::UNAUTHORIZED, "Invalid credentials")
        }
        Err(SessionCreateError::MfaFailed) => {
            error(StatusCode::PRECONDITION_FAILED, "Invalid code")
        }
        Err(SessionCreateError::UserDisabled) => error(StatusCode::FORBIDDEN, "User disabled"),
        Err(SessionCreateError::Recaptcha) => {
            error(StatusCode::PRECONDITION_FAILED, "Recaptcha failed")
        }
        Err(SessionCreateError::Other(err)) => internal_server_error(err),
    }
}

async fn impersonate(
    session_service: State<Arc<impl SessionService>>,
    token: ApiToken,
    Path(user_id): Path<UserId>,
) -> Response {
    match session_service.impersonate(&token.0, user_id).await {
        Ok(login) => (StatusCode::CREATED, Json(ApiLogin::from(login))).into_response(),
        Err(SessionImpersonateError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(SessionImpersonateError::Auth(err)) => auth_error(err),
        Err(SessionImpersonateError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh(
    session_service: State<Arc<impl SessionService>>,
    Json(RefreshRequest { refresh_token }): Json<RefreshRequest>,
) -> Response {
    match session_service.refresh_session(&refresh_token).await {
        Ok(login) => Json(ApiLogin::from(login)).into_response(),
        Err(SessionRefreshError::InvalidRefreshToken) => {
            error(StatusCode::UNAUTHORIZED, "Invalid refresh token")
        }
        Err(SessionRefreshError::Other(err)) => internal_server_error(err),
    }
}

async fn delete(
    session_service: State<Arc<impl SessionService>>,
    token: ApiToken,
    Path((user_id, session_id)): Path<(ApiUserIdOrSelf, SessionId)>,
) -> Response {
    match session_service
        .delete_session(&token.0, user_id.into(), session_id)
        .await
    {
        Ok(()) => Json(true).into_response(),
        Err(SessionDeleteError::NotFound) => error(StatusCode::NOT_FOUND, "Session not found"),
        Err(SessionDeleteError::Auth(err)) => auth_error(err),
        Err(SessionDeleteError::Other(err)) => internal_server_error(err),
    }
}

async fn delete_current(
    session_service: State<Arc<impl SessionService>>,
    token: ApiToken,
) -> Response {
    match session_service.delete_current_session(&token.0).await {
        Ok(()) => Json(true).into_response(),
        Err(SessionDeleteCurrentError::Auth(err)) => auth_error(err),
        Err(SessionDeleteCurrentError::Other(err)) => internal_server_error(err),
    }
}

async fn delete_by_user(
    session_service: State<Arc<impl SessionService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
) -> Response {
    match session_service
        .delete_by_user(&token.0, user_id.into())
        .await
    {
        Ok(()) => Json(true).into_response(),
        Err(SessionDeleteByUserError::Auth(err)) => auth_error(err),
        Err(SessionDeleteByUserError::Other(err)) => internal_server_error(err),
    }
}
