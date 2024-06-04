use thiserror::Error;

use crate::{session::Session, user::UserComposite};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Login {
    pub user_composite: UserComposite,
    pub session: Session,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error(transparent)]
    Authenticate(#[from] AuthenticateError),
    #[error(transparent)]
    Authorize(#[from] AuthorizeError),
}

#[derive(Debug, Error)]
pub enum AuthenticateError {
    #[error("The access token is invalid or has expired.")]
    InvalidToken,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum AuthorizeError {
    #[error("The user is not an administrator.")]
    Admin,
    #[error("The user's email address is not verified.")]
    EmailVerified,
}
