use std::collections::HashMap;

use academy_core_auth_contracts::MockAuthService;
use academy_core_oauth2_contracts::{
    create_link::MockOAuth2CreateLinkService, login::MockOAuth2LoginService,
};
use academy_demo::oauth2::{TEST_OAUTH2_PROVIDER, TEST_OAUTH2_PROVIDER_ID};
use academy_extern_contracts::oauth2::MockOAuth2ApiService;
use academy_persistence_contracts::{
    oauth2::MockOAuth2Repository, user::MockUserRepository, MockDatabase, MockTransaction,
};

use crate::{OAuth2ServiceConfig, OAuth2ServiceImpl};

mod create_link;
mod delete_link;
mod list_links;
mod list_providers;

type Sut = OAuth2ServiceImpl<
    MockDatabase,
    MockAuthService<MockTransaction>,
    MockOAuth2ApiService,
    MockUserRepository<MockTransaction>,
    MockOAuth2Repository<MockTransaction>,
    MockOAuth2CreateLinkService<MockTransaction>,
    MockOAuth2LoginService,
>;

impl Default for OAuth2ServiceConfig {
    fn default() -> Self {
        Self {
            providers: HashMap::from([(
                TEST_OAUTH2_PROVIDER_ID.clone(),
                TEST_OAUTH2_PROVIDER.clone(),
            )])
            .into(),
        }
    }
}
