use std::future::Future;

use academy_models::{
    email_address::EmailAddress,
    user::{UserComposite, UserId},
};
use auth::InternalAuthError;
use thiserror::Error;

pub mod auth;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait InternalService: Send + Sync + 'static {
    fn get_user(
        &self,
        token: &str,
        user_id: UserId,
    ) -> impl Future<Output = Result<UserComposite, InternalGetUserError>> + Send;

    fn get_user_by_email(
        &self,
        token: &str,
        email: EmailAddress,
    ) -> impl Future<Output = Result<UserComposite, InternalGetUserByEmailError>> + Send;
}

#[derive(Debug, Error)]
pub enum InternalGetUserError {
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Auth(#[from] InternalAuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum InternalGetUserByEmailError {
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Auth(#[from] InternalAuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
