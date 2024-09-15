use std::future::Future;

use academy_models::{
    auth::AuthError,
    mfa::{MfaRecoveryCode, TotpCode, TotpSetup},
    user::UserIdOrSelf,
};
use thiserror::Error;

pub mod authenticate;
pub mod disable;
pub mod recovery;
pub mod totp_device;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaFeatureService: Send + Sync + 'static {
    fn initialize(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> impl Future<Output = Result<TotpSetup, MfaInitializeError>> + Send;

    fn enable(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        code: TotpCode,
    ) -> impl Future<Output = Result<MfaRecoveryCode, MfaEnableError>> + Send;

    fn disable(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> impl Future<Output = Result<(), MfaDisableError>> + Send;
}

#[derive(Debug, Error)]
pub enum MfaInitializeError {
    #[error("The user has already enabled mfa.")]
    AlreadyEnabled,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum MfaEnableError {
    #[error("The user has already enabled mfa.")]
    AlreadyEnabled,
    #[error("Mfa has not been initialized.")]
    NotInitialized,
    #[error("The totp code in incorrect.")]
    InvalidCode,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum MfaDisableError {
    #[error("The user has not enabled mfa.")]
    NotEnabled,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
