use std::future::Future;

use academy_models::{
    auth::{AuthError, Login},
    oauth2::{
        OAuth2Link, OAuth2LinkId, OAuth2Login, OAuth2ProviderSummary, OAuth2RegistrationToken,
    },
    session::DeviceName,
    user::UserIdOrSelf,
};
use thiserror::Error;

pub mod create_link;
pub mod login;

pub trait OAuth2Service: Send + Sync + 'static {
    fn list_providers(&self) -> Vec<OAuth2ProviderSummary>;

    fn list_links(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> impl Future<Output = Result<Vec<OAuth2Link>, OAuth2ListLinksError>> + Send;

    fn create_link(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        login: OAuth2Login,
    ) -> impl Future<Output = Result<OAuth2Link, OAuth2CreateLinkError>> + Send;

    fn delete_link(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        link_id: OAuth2LinkId,
    ) -> impl Future<Output = Result<(), OAuth2DeleteLinkError>> + Send;

    fn create_session(
        &self,
        login: OAuth2Login,
        device_name: Option<DeviceName>,
    ) -> impl Future<Output = Result<OAuth2CreateSessionResponse, OAuth2CreateSessionError>> + Send;
}

#[derive(Debug, Error)]
pub enum OAuth2ListLinksError {
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum OAuth2CreateLinkError {
    #[error("The provider does not exist.")]
    InvalidProvider,
    #[error("The authorization code is invalid.")]
    InvalidCode,
    #[error("The remote user has already been linked.")]
    RemoteAlreadyLinked,
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum OAuth2DeleteLinkError {
    #[error("The link does not exist.")]
    NotFound,
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuth2CreateSessionResponse {
    Login(Box<Login>),
    RegistrationToken(OAuth2RegistrationToken),
}

#[derive(Debug, Error)]
pub enum OAuth2CreateSessionError {
    #[error("The provider does not exist.")]
    InvalidProvider,
    #[error("The authorization code is invalid.")]
    InvalidCode,
    #[error("The user account has been disabled.")]
    UserDisabled,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub fn oauth2_registration_cache_key(registration_token: &OAuth2RegistrationToken) -> String {
    format!("oauth2_registration:{}", **registration_token)
}
