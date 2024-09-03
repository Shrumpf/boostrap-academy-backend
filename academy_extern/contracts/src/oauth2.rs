use std::future::Future;

use thiserror::Error;
use url::Url;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait OAuth2Service: Send + Sync + 'static {
    fn generate_auth_url(&self, provider: OAuth2Provider) -> Url;

    fn resolve_code(
        &self,
        provider: OAuth2Provider,
        code: String,
    ) -> impl Future<Output = Result<OAuth2UserInfo, OAuth2ResolveCodeError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2Provider {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: Url,
    pub token_url: Url,
    pub redirect_url: Url,
    pub userinfo_url: Url,
    pub userinfo_id_key: String,
    pub userinfo_name_key: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2UserInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Error)]
pub enum OAuth2ResolveCodeError {
    #[error("The authorization code is invalid.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl MockOAuth2Service {
    pub fn with_resolve_code(
        mut self,
        provider: OAuth2Provider,
        code: String,
        result: Result<OAuth2UserInfo, OAuth2ResolveCodeError>,
    ) -> Self {
        self.expect_resolve_code()
            .once()
            .with(
                mockall::predicate::eq(provider),
                mockall::predicate::eq(code),
            )
            .return_once(|_, _| Box::pin(std::future::ready(result)));
        self
    }
}
