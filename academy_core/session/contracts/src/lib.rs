use std::future::Future;

use academy_models::{
    auth::{AuthError, Login},
    mfa::MfaAuthentication,
    session::{DeviceName, Session, SessionId},
    user::{UserId, UserIdOrSelf, UserNameOrEmailAddress, UserPassword},
    RecaptchaResponse,
};
use thiserror::Error;

pub mod commands;
pub mod failed_auth_count;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait SessionService: Send + Sync + 'static {
    /// Returns the currently authenticated session.
    fn get_current_session(
        &self,
        token: &str,
    ) -> impl Future<Output = Result<Session, SessionGetCurrentError>> + Send;

    /// Returns all sessions of a given user.
    ///
    /// Can only be used by administrators, if not used on the authenticated
    /// user.
    fn list_by_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> impl Future<Output = Result<Vec<Session>, SessionListByUserError>> + Send;

    /// Creates a new session using a username and password.
    fn create_session(
        &self,
        cmd: SessionCreateCommand,
        recaptcha_response: Option<RecaptchaResponse>,
    ) -> impl Future<Output = Result<Login, SessionCreateError>> + Send;

    /// Impersonate a user by creating a new session for them.
    ///
    /// Can only be used by administrators.
    fn impersonate(
        &self,
        token: &str,
        user_id: UserId,
    ) -> impl Future<Output = Result<Login, SessionImpersonateError>> + Send;

    /// Refreshes a session using a refresh token.
    ///
    /// This will generate a new access and refresh token pair and invalidate
    /// the previous one.
    fn refresh_session(
        &self,
        refresh_token: &str,
    ) -> impl Future<Output = Result<Login, SessionRefreshError>> + Send;

    /// Deletes a session and invalidates the access and refresh tokens
    /// associated with it.
    ///
    /// Can only be used by administrators, if not used on the authenticated
    /// user.
    fn delete_session(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        session_id: SessionId,
    ) -> impl Future<Output = Result<(), SessionDeleteError>> + Send;

    /// Deletes the currently authenticated session and invalidates its access
    /// and refresh tokens.
    fn delete_current_session(
        &self,
        token: &str,
    ) -> impl Future<Output = Result<(), SessionDeleteCurrentError>> + Send;

    /// Deletes all sessions of a given user and invalidates all access and
    /// refresh tokens associated with them.
    ///
    /// Can only be used by administrators, if not used on the authenticated
    /// user.
    fn delete_by_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> impl Future<Output = Result<(), SessionDeleteByUserError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCreateCommand {
    pub name_or_email: UserNameOrEmailAddress,
    pub password: UserPassword,
    pub mfa: MfaAuthentication,
    pub device_name: Option<DeviceName>,
}

#[derive(Debug, Error)]
pub enum SessionGetCurrentError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SessionListByUserError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SessionCreateError {
    #[error("The user does not exist or the password is incorrect.")]
    InvalidCredentials,
    #[error("The user has mfa enabled but no valid authentication was provided.")]
    MfaFailed,
    #[error("The user account has been disabled.")]
    UserDisabled,
    #[error("Invalid recaptcha response")]
    Recaptcha,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SessionImpersonateError {
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SessionRefreshError {
    #[error("The refresh token is invalid or has expired.")]
    InvalidRefreshToken,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SessionDeleteError {
    #[error("The session does not exist.")]
    NotFound,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SessionDeleteCurrentError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SessionDeleteByUserError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
