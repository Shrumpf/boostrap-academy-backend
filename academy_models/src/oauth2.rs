use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2Provider {
    pub name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: Url,
    pub token_url: Url,
    pub userinfo_url: Url,
    pub userinfo_id_key: String,
    pub userinfo_name_key: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2ProviderSummary {
    pub id: String,
    pub name: String,
    pub auth_url: Url,
}
