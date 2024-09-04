use std::future::Future;

use academy_models::oauth2::{OAuth2Login, OAuth2UserInfo};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait OAuth2LoginService: Send + Sync + 'static {
    fn invoke(
        &self,
        login: OAuth2Login,
    ) -> impl Future<Output = Result<OAuth2UserInfo, OAuth2LoginServiceError>> + Send;
}

#[derive(Debug, Error)]
pub enum OAuth2LoginServiceError {
    #[error("The provider does not exist.")]
    InvalidProvider,
    #[error("The authorization code is invalid.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl MockOAuth2LoginService {
    pub fn with_invoke(
        mut self,
        login: OAuth2Login,
        result: Result<OAuth2UserInfo, OAuth2LoginServiceError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(mockall::predicate::eq(login))
            .return_once(|_| Box::pin(std::future::ready(result)));
        self
    }
}
