use std::future::Future;

use academy_models::oauth2::{OAuth2AuthorizationCode, OAuth2Provider, OAuth2UserInfo};
use thiserror::Error;
use url::Url;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait OAuth2ApiService: Send + Sync + 'static {
    fn generate_auth_url(&self, provider: &OAuth2Provider) -> Url;

    fn resolve_code(
        &self,
        provider: OAuth2Provider,
        code: OAuth2AuthorizationCode,
        redirect_url: Url,
    ) -> impl Future<Output = Result<OAuth2UserInfo, OAuth2ResolveCodeError>> + Send;
}

#[derive(Debug, Error)]
pub enum OAuth2ResolveCodeError {
    #[error("The authorization code is invalid.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl MockOAuth2ApiService {
    pub fn with_generate_auth_url(mut self, provider: OAuth2Provider, result: Url) -> Self {
        self.expect_generate_auth_url()
            .once()
            .with(mockall::predicate::eq(provider))
            .return_once(|_| result);
        self
    }

    pub fn with_resolve_code(
        mut self,
        provider: OAuth2Provider,
        code: OAuth2AuthorizationCode,
        redirect_url: Url,
        result: Result<OAuth2UserInfo, OAuth2ResolveCodeError>,
    ) -> Self {
        self.expect_resolve_code()
            .once()
            .with(
                mockall::predicate::eq(provider),
                mockall::predicate::eq(code),
                mockall::predicate::eq(redirect_url),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(result)));
        self
    }
}
