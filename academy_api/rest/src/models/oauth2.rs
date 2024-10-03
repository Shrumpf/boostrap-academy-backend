use academy_models::{
    oauth2::{
        OAuth2AuthorizationCode, OAuth2Link, OAuth2LinkId, OAuth2Login, OAuth2ProviderId,
        OAuth2ProviderName, OAuth2ProviderSummary, OAuth2RemoteUserName,
    },
    url::Url,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, JsonSchema)]
pub struct ApiOAuth2ProviderSummary {
    /// OAuth2 provider ID
    pub id: OAuth2ProviderId,
    /// Display name
    pub name: OAuth2ProviderName,
    /// Remote authorize endpoint URL *without* `state` and `redirect_uri`
    /// parameters
    pub authorize_url: Url,
}

impl From<OAuth2ProviderSummary> for ApiOAuth2ProviderSummary {
    fn from(value: OAuth2ProviderSummary) -> Self {
        Self {
            id: value.id,
            name: value.name,
            authorize_url: value.auth_url,
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ApiOAuth2Link {
    /// OAuth2 link ID
    pub id: OAuth2LinkId,
    /// OAuth2 provider ID
    pub provider_id: OAuth2ProviderId,
    /// Display name of the remote user account
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

#[derive(Deserialize, JsonSchema)]
pub struct ApiOAuth2Login {
    /// OAuth2 provider ID
    pub provider_id: OAuth2ProviderId,
    /// Authorization code returned by the OAuth2 provider
    pub code: OAuth2AuthorizationCode,
    /// Redirect URI that was used for this authentication.
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
