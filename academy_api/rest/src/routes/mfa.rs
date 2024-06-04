use std::sync::Arc;

use academy_core_mfa_contracts::{MfaDisableError, MfaEnableError, MfaInitializeError, MfaService};
use academy_models::mfa::TotpCode;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use serde::Deserialize;

use super::{auth_error, error, internal_server_error};
use crate::{extractors::auth::ApiToken, models::user::ApiUserIdOrSelf};

pub fn router(service: Arc<impl MfaService>) -> Router<()> {
    Router::new()
        .route(
            "/auth/users/:user_id/mfa",
            routing::post(initialize).put(enable).delete(disable),
        )
        .with_state(service)
}

async fn initialize(
    service: State<Arc<impl MfaService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
) -> Response {
    match service.initialize(&token.0, user_id.into()).await {
        Ok(setup) => Json(setup.secret).into_response(),
        Err(MfaInitializeError::AlreadyEnabled) => {
            error(StatusCode::CONFLICT, "MFA already enabled")
        }
        Err(MfaInitializeError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(MfaInitializeError::Auth(err)) => auth_error(err),
        Err(MfaInitializeError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct EnableRequest {
    code: TotpCode,
}

async fn enable(
    service: State<Arc<impl MfaService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
    Json(EnableRequest { code }): Json<EnableRequest>,
) -> Response {
    match service.enable(&token.0, user_id.into(), code).await {
        Ok(recovery_code) => Json(recovery_code).into_response(),
        Err(MfaEnableError::AlreadyEnabled) => error(StatusCode::CONFLICT, "MFA already enabled"),
        Err(MfaEnableError::NotInitialized) => {
            error(StatusCode::PRECONDITION_FAILED, "MFA not initialized")
        }
        Err(MfaEnableError::InvalidCode) => error(StatusCode::PRECONDITION_FAILED, "Invalid code"),
        Err(MfaEnableError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(MfaEnableError::Auth(err)) => auth_error(err),
        Err(MfaEnableError::Other(err)) => internal_server_error(err),
    }
}

async fn disable(
    service: State<Arc<impl MfaService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
) -> Response {
    match service.disable(&token.0, user_id.into()).await {
        Ok(()) => Json(true).into_response(),
        Err(MfaDisableError::NotEnabled) => {
            error(StatusCode::PRECONDITION_FAILED, "MFA not enabled")
        }
        Err(MfaDisableError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(MfaDisableError::Auth(err)) => auth_error(err),
        Err(MfaDisableError::Other(err)) => internal_server_error(err),
    }
}
