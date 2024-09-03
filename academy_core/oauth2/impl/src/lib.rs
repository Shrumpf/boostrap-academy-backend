use std::{collections::HashMap, sync::Arc};

use academy_core_oauth2_contracts::OAuth2Service;
use academy_di::Build;
use academy_extern_contracts::oauth2::OAuth2ApiService;
use academy_models::oauth2::{OAuth2Provider, OAuth2ProviderSummary};

#[derive(Debug, Clone, Build)]
pub struct OAuth2ServiceImpl<OAuth2Api> {
    oauth2_api: OAuth2Api,
    config: OAuth2ServiceConfig,
}

#[derive(Debug, Clone)]
pub struct OAuth2ServiceConfig {
    pub providers: Arc<HashMap<String, OAuth2Provider>>,
}

impl<OAuth2Api> OAuth2Service for OAuth2ServiceImpl<OAuth2Api>
where
    OAuth2Api: OAuth2ApiService,
{
    fn list_providers(&self) -> Vec<OAuth2ProviderSummary> {
        self.config
            .providers
            .iter()
            .map(|(id, provider)| OAuth2ProviderSummary {
                id: id.clone(),
                name: provider.name.clone(),
                auth_url: self.oauth2_api.generate_auth_url(provider),
            })
            .collect()
    }
}
