use academy_auth_contracts::internal::AuthInternalService;
use academy_di::Build;
use academy_extern_contracts::internal::InternalApiService;
use academy_models::user::UserId;
use url::Url;

use crate::http::HttpClient;

#[derive(Debug, Clone, Build)]
pub struct InternalApiServiceImpl<AuthInternal> {
    auth_internal: AuthInternal,
    config: InternalApiServiceConfig,
    #[state]
    http: HttpClient,
}

#[derive(Debug, Clone)]
pub struct InternalApiServiceConfig {
    pub shop_url: Url,
}

impl<AuthInternal> InternalApiService for InternalApiServiceImpl<AuthInternal>
where
    AuthInternal: AuthInternalService,
{
    async fn release_coins(&self, user_id: UserId) -> anyhow::Result<()> {
        self.http
            .put(self.config.shop_url.join(&format!(
                "_internal/coins/{}/withheld",
                user_id.hyphenated()
            ))?)
            .bearer_auth(self.auth_internal.issue_token("shop")?)
            .send()
            .await?
            .error_for_status()
            .map(|_| ())
            .map_err(Into::into)
    }
}
