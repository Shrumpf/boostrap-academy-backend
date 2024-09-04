use std::future::Future;

use academy_models::{
    auth::AuthError,
    oauth2::{OAuth2Link, OAuth2LinkId, OAuth2Login, OAuth2ProviderSummary},
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
