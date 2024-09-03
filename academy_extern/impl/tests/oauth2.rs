use std::collections::HashMap;

use academy_extern_contracts::oauth2::{OAuth2ApiService, OAuth2ResolveCodeError, OAuth2UserInfo};
use academy_extern_impl::oauth2::OAuth2ApiServiceImpl;
use academy_models::oauth2::OAuth2Provider;
use academy_utils::assert_matches;
use url::Url;

#[tokio::test]
async fn oauth2() {
    let provider = get_provider();

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut url = provider.auth_url.clone();
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &provider.client_id)
        .append_pair("state", "thestate")
        .append_pair("redirect_uri", redirect_url().as_str())
        .finish();
    let form = HashMap::from([("id", "userid123"), ("name", "theremoteusername")]);
    let response = client
        .post(url)
        .form(&form)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let url = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<Url>()
        .unwrap();
    let code = url.query_pairs().find(|(k, _)| *k == "code").unwrap().1;
    let state = url.query_pairs().find(|(k, _)| *k == "state").unwrap().1;
    assert_eq!(state, "thestate");

    let sut = OAuth2ApiServiceImpl::default();

    let result = sut
        .resolve_code(provider.clone(), code.into(), redirect_url())
        .await
        .unwrap();
    assert_eq!(
        result,
        OAuth2UserInfo {
            id: "userid123".into(),
            name: "theremoteusername".into()
        }
    );

    let result = sut
        .resolve_code(provider, "invalidcode".into(), redirect_url())
        .await;
    assert_matches!(result, Err(OAuth2ResolveCodeError::InvalidCode));
}

fn get_provider() -> OAuth2Provider {
    let base_url = Url::parse("http://localhost:8002").unwrap();

    OAuth2Provider {
        name: "test".into(),
        client_id: "client-id".into(),
        client_secret: Some("client-secret".into()),
        auth_url: base_url.join("oauth2/authorize").unwrap(),
        token_url: base_url.join("oauth2/token").unwrap(),
        userinfo_url: base_url.join("user").unwrap(),
        userinfo_id_key: "id".into(),
        userinfo_name_key: "name".into(),
        scopes: vec![],
    }
}

fn redirect_url() -> Url {
    Url::parse("http://localhost/oauth2/callback").unwrap()
}
