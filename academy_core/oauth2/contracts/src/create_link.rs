use std::future::Future;

use academy_models::{
    oauth2::{OAuth2Link, OAuth2ProviderId, OAuth2UserInfo},
    user::UserId,
};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait OAuth2CreateLinkService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        provider_id: OAuth2ProviderId,
        remote_user: OAuth2UserInfo,
    ) -> impl Future<Output = Result<OAuth2Link, OAuth2CreateLinkServiceError>> + Send;
}

#[derive(Debug, Error)]
pub enum OAuth2CreateLinkServiceError {
    #[error("The remote user has already been linked.")]
    RemoteAlreadyLinked,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockOAuth2CreateLinkService<Txn> {
    pub fn with_invoke(
        mut self,
        user_id: UserId,
        provider_id: OAuth2ProviderId,
        remote_user: OAuth2UserInfo,
        result: Result<OAuth2Link, OAuth2CreateLinkServiceError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(provider_id),
                mockall::predicate::eq(remote_user),
            )
            .return_once(|_, _, _, _| Box::pin(std::future::ready(result)));
        self
    }
}
