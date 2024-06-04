use std::time::Duration;

use academy_cache_contracts::MockCacheService;
use academy_core_auth_contracts::commands::invalidate_access_token::MockAuthInvalidateAccessTokenCommandService;
use academy_persistence_contracts::{session::MockSessionRepository, user::MockUserRepository};
use academy_shared_contracts::{
    hash::MockHashService, jwt::MockJwtService, password::MockPasswordService,
    secret::MockSecretService, time::MockTimeService,
};

use crate::{AuthServiceConfig, AuthServiceImpl};

mod authenticate;
mod authenticate_by_password;
mod authenticate_by_refresh_token;
mod invalidate_access_token;
mod invalidate_access_tokens;
mod issue_tokens;

type Sut = AuthServiceImpl<
    MockJwtService,
    MockSecretService,
    MockTimeService,
    MockHashService,
    MockPasswordService,
    MockUserRepository<()>,
    MockSessionRepository<()>,
    MockCacheService,
    MockAuthInvalidateAccessTokenCommandService,
>;

impl Default for AuthServiceConfig {
    fn default() -> Self {
        Self {
            access_token_ttl: Duration::from_secs(120),
            refresh_token_ttl: Duration::from_secs(30 * 24 * 3600),
            refresh_token_length: 64,
        }
    }
}
