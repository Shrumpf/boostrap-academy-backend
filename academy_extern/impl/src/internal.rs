use std::time::Duration;

use academy_di::Build;
use academy_extern_contracts::internal::InternalApiService;
use academy_models::user::UserId;
use academy_shared_contracts::jwt::JwtService;
use serde::Serialize;
use url::Url;

use crate::http::HttpClient;

#[derive(Debug, Clone, Build)]
pub struct InternalApiServiceImpl<Jwt> {
    jwt: Jwt,
    config: InternalApiServiceConfig,
    #[state]
    http: HttpClient,
}

#[derive(Debug, Clone)]
pub struct InternalApiServiceConfig {
    pub jwt_ttl: Duration,
    pub shop_url: Url,
}

impl<Jwt> InternalApiService for InternalApiServiceImpl<Jwt>
where
    Jwt: JwtService,
{
    async fn release_coins(&self, user_id: UserId) -> anyhow::Result<()> {
        self.http
            .put(self.config.shop_url.join(&format!(
                "_internal/coins/{}/withheld",
                user_id.hyphenated()
            ))?)
            .bearer_auth(self.auth("shop")?)
            .send()
            .await?
            .error_for_status()
            .map(|_| ())
            .map_err(Into::into)
    }
}

impl<Jwt> InternalApiServiceImpl<Jwt>
where
    Jwt: JwtService,
{
    fn auth(&self, aud: &'static str) -> anyhow::Result<String> {
        self.jwt
            .sign(InternalAuthTokenData { aud }, self.config.jwt_ttl)
    }
}

#[derive(Debug, Serialize)]
struct InternalAuthTokenData {
    aud: &'static str,
}
