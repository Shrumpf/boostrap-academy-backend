use academy_models::oauth2::OAuth2ProviderSummary;
use serde::Serialize;
use url::Url;

#[derive(Debug, Serialize)]
pub struct ApiOAuth2ProviderSummary {
    pub id: String,
    pub name: String,
    pub auth_url: Url,
}

impl From<OAuth2ProviderSummary> for ApiOAuth2ProviderSummary {
    fn from(value: OAuth2ProviderSummary) -> Self {
        Self {
            id: value.id,
            name: value.name,
            auth_url: value.auth_url,
        }
    }
}
