use academy_models::oauth2::{
    OAuth2AuthorizationCode, OAuth2Link, OAuth2LinkId, OAuth2Login, OAuth2ProviderId,
    OAuth2ProviderName, OAuth2ProviderSummary, OAuth2RemoteUserName,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize)]
pub struct ApiOAuth2ProviderSummary {
    pub id: OAuth2ProviderId,
    pub name: OAuth2ProviderName,
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

#[derive(Debug, Serialize)]
pub struct ApiOAuth2Link {
    pub id: OAuth2LinkId,
    pub provider_id: OAuth2ProviderId,
    pub display_name: OAuth2RemoteUserName,
}

impl From<OAuth2Link> for ApiOAuth2Link {
    fn from(value: OAuth2Link) -> Self {
        Self {
            id: value.id,
            provider_id: value.provider_id,
            display_name: value.remote_user.name,
        }
    }
}

#[derive(Deserialize)]
pub struct ApiOAuth2Login {
    pub provider_id: OAuth2ProviderId,
    pub code: OAuth2AuthorizationCode,
    pub redirect_uri: Url,
}

impl From<ApiOAuth2Login> for OAuth2Login {
    fn from(value: ApiOAuth2Login) -> Self {
        Self {
            provider_id: value.provider_id,
            code: value.code,
            redirect_uri: value.redirect_uri,
        }
    }
}
