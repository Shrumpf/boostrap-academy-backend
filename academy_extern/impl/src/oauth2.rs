use std::collections::HashMap;

use academy_di::Build;
use academy_extern_contracts::oauth2::{
    OAuth2Provider, OAuth2ResolveCodeError, OAuth2Service, OAuth2UserInfo,
};
use academy_utils::Apply;
use anyhow::anyhow;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl,
    RequestTokenError, TokenResponse, TokenUrl,
};
use url::Url;

use crate::http::{HttpClient, USER_AGENT};

#[derive(Debug, Clone, Build, Default)]
pub struct OAuth2ServiceImpl {
    #[state]
    http: HttpClient,
}

impl OAuth2Service for OAuth2ServiceImpl {
    fn generate_auth_url(&self, provider: OAuth2Provider) -> Url {
        let mut url = provider.auth_url;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &provider.client_id)
            .apply_if(!provider.scopes.is_empty(), |q| {
                let mut it = provider.scopes.into_iter();
                let mut scopes = it.next().unwrap();
                for scope in it {
                    scopes.push(' ');
                    scopes.push_str(&scope);
                }
                q.append_pair("scope", &scopes)
            })
            .finish();
        url
    }

    async fn resolve_code(
        &self,
        provider: OAuth2Provider,
        code: String,
    ) -> Result<OAuth2UserInfo, OAuth2ResolveCodeError> {
        let client = BasicClient::new(
            ClientId::new(provider.client_id),
            provider.client_secret.map(ClientSecret::new),
            AuthUrl::from_url(provider.auth_url),
            Some(TokenUrl::from_url(provider.token_url)),
        )
        .set_redirect_uri(RedirectUrl::from_url(provider.redirect_url));

        let response = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(http_client)
            .await
            .map_err(|err| match err {
                RequestTokenError::ServerResponse(_) | RequestTokenError::Parse(_, _) => {
                    OAuth2ResolveCodeError::InvalidCode
                }
                err => OAuth2ResolveCodeError::Other(err.into()),
            })?;

        let access_token = response.access_token().secret();

        let userinfo = self
            .http
            .get(provider.userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|err| OAuth2ResolveCodeError::Other(err.into()))?
            .error_for_status()
            .map_err(|err| OAuth2ResolveCodeError::Other(err.into()))?
            .json::<HashMap<String, serde_json::Value>>()
            .await
            .map_err(|err| OAuth2ResolveCodeError::Other(err.into()))?;

        let id = match userinfo.get(&provider.userinfo_id_key) {
            Some(serde_json::Value::Number(id)) => id.to_string(),
            Some(serde_json::Value::String(id)) => id.to_owned(),
            Some(x) => return Err(anyhow!("Invalid user id: {x}").into()),
            None => return Err(anyhow!("User id missing").into()),
        };

        let name = match userinfo.get(&provider.userinfo_name_key) {
            Some(serde_json::Value::String(name)) => name.clone(),
            Some(x) => return Err(anyhow!("Invalid username: {x}").into()),
            None => return Err(anyhow!("Username missing").into()),
        };

        Ok(OAuth2UserInfo { id, name })
    }
}

async fn http_client(
    mut request: oauth2::HttpRequest,
) -> Result<oauth2::HttpResponse, oauth2::reqwest::AsyncHttpClientError> {
    request.headers.insert(
        oauth2::http::header::USER_AGENT,
        oauth2::http::HeaderValue::from_static(USER_AGENT),
    );
    oauth2::reqwest::async_http_client(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_auth_url_with_scopes() {
        // Arrange
        let provider = make_provider();

        let sut = OAuth2ServiceImpl::default();

        // Act
        let result = sut.generate_auth_url(provider);

        // Assert
        assert_eq!(result.as_str(), "https://oauth2.provider/auth?response_type=code&client_id=the-client-id&scope=foo+bar+baz");
    }

    #[test]
    fn generate_auth_url_without_scopes() {
        // Arrange
        let provider = OAuth2Provider {
            scopes: Vec::new(),
            ..make_provider()
        };

        let sut = OAuth2ServiceImpl::default();

        // Act
        let result = sut.generate_auth_url(provider);

        // Assert
        assert_eq!(
            result.as_str(),
            "https://oauth2.provider/auth?response_type=code&client_id=the-client-id"
        );
    }

    fn make_provider() -> OAuth2Provider {
        OAuth2Provider {
            client_id: "the-client-id".into(),
            client_secret: None,
            auth_url: "https://oauth2.provider/auth".parse().unwrap(),
            token_url: "http://test".parse().unwrap(),
            redirect_url: "http://test".parse().unwrap(),
            userinfo_url: "http://test".parse().unwrap(),
            userinfo_id_key: String::new(),
            userinfo_name_key: String::new(),
            scopes: ["foo", "bar", "baz"].map(Into::into).into(),
        }
    }
}
