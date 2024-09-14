use academy_core_auth_contracts::refresh_token::AuthRefreshTokenService;
use academy_di::Build;
use academy_models::session::SessionRefreshTokenHash;
use academy_shared_contracts::{hash::HashService, secret::SecretService};

use crate::AuthServiceConfig;

#[derive(Debug, Clone, Build)]
#[cfg_attr(test, derive(Default))]
pub struct AuthRefreshTokenServiceImpl<Secret, Hash> {
    secret: Secret,
    hash: Hash,
    config: AuthServiceConfig,
}

impl<Secret, Hash> AuthRefreshTokenService for AuthRefreshTokenServiceImpl<Secret, Hash>
where
    Secret: SecretService,
    Hash: HashService,
{
    fn issue(&self) -> (String, SessionRefreshTokenHash) {
        let refresh_token = self.secret.generate(self.config.refresh_token_length);
        let refresh_token_hash = self.hash(&refresh_token);
        (refresh_token, refresh_token_hash)
    }

    fn hash(&self, refresh_token: &str) -> SessionRefreshTokenHash {
        self.hash.sha256(refresh_token.as_bytes()).into()
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::SHA256HASH1;
    use academy_shared_contracts::{hash::MockHashService, secret::MockSecretService};

    use super::*;

    type Sut = AuthRefreshTokenServiceImpl<MockSecretService, MockHashService>;

    #[test]
    fn issue() {
        // Arrange
        let config = AuthServiceConfig::default();

        let refresh_token = "the new refresh token";

        let secret = MockSecretService::new()
            .with_generate(config.refresh_token_length, refresh_token.into());
        let hash = MockHashService::new().with_sha256(refresh_token.into(), *SHA256HASH1);

        let sut = AuthRefreshTokenServiceImpl {
            secret,
            hash,
            config,
        };

        // Act
        let result = sut.issue();

        // Assert
        assert_eq!(result, (refresh_token.into(), (*SHA256HASH1).into()));
    }

    #[test]
    fn hash() {
        // Arrange
        let refresh_token = "the refresh token";

        let hash = MockHashService::new().with_sha256(refresh_token.into(), *SHA256HASH1);

        let sut = AuthRefreshTokenServiceImpl {
            hash,
            ..Sut::default()
        };

        // Act
        let result = sut.hash(refresh_token);

        // Assert
        assert_eq!(result, (*SHA256HASH1).into());
    }
}
