use academy_cache_contracts::CacheService;
use academy_core_auth_contracts::commands::invalidate_access_token::AuthInvalidateAccessTokenCommandService;
use academy_di::Build;
use academy_models::session::SessionRefreshTokenHash;

use crate::{access_token_invalidated_key, AuthServiceConfig};

#[derive(Debug, Clone, Build)]
pub struct AuthInvalidateAccessTokenCommandServiceImpl<Cache> {
    cache: Cache,
    config: AuthServiceConfig,
}

impl<Cache> AuthInvalidateAccessTokenCommandService
    for AuthInvalidateAccessTokenCommandServiceImpl<Cache>
where
    Cache: CacheService,
{
    async fn invoke(&self, refresh_token_hash: SessionRefreshTokenHash) -> anyhow::Result<()> {
        self.cache
            .set(
                &access_token_invalidated_key(refresh_token_hash),
                &(),
                Some(self.config.access_token_ttl),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use academy_cache_contracts::MockCacheService;
    use academy_demo::{SHA256HASH1, SHA256HASH1_HEX};

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let config = AuthServiceConfig::default();

        let cache = MockCacheService::new().with_set(
            format!("access_token_invalidated:{SHA256HASH1_HEX}"),
            (),
            Some(config.access_token_ttl),
        );

        let sut = AuthInvalidateAccessTokenCommandServiceImpl { config, cache };

        // Act
        let result = sut.invoke((*SHA256HASH1).into()).await;

        // Assert
        result.unwrap();
    }
}
