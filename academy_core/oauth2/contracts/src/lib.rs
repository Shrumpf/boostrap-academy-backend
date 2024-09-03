use academy_models::oauth2::OAuth2ProviderSummary;

pub trait OAuth2Service: Send + Sync + 'static {
    fn list_providers(&self) -> Vec<OAuth2ProviderSummary>;
}
