use academy_core_internal_contracts::auth::{InternalAuthError, InternalAuthService};
use academy_di::Build;
use academy_shared_contracts::jwt::JwtService;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Build)]
pub struct InternalAuthServiceImpl<Jwt> {
    jwt: Jwt,
}

impl<Jwt> InternalAuthService for InternalAuthServiceImpl<Jwt>
where
    Jwt: JwtService,
{
    fn authenticate(&self, token: &str, audience: &str) -> Result<(), InternalAuthError> {
        self.jwt
            .verify::<InternalAuthTokenData>(token)
            .ok()
            .filter(|data| data.aud == audience)
            .map(|_| ())
            .ok_or(InternalAuthError::InvalidToken)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalAuthTokenData {
    aud: String,
}

#[cfg(test)]
mod tests {
    use academy_shared_contracts::jwt::{MockJwtService, VerifyJwtError};
    use academy_utils::assert_matches;

    use super::*;

    #[test]
    fn ok() {
        // Arrange
        let jwt = MockJwtService::new()
            .with_verify("token", Ok(InternalAuthTokenData { aud: "auth".into() }));

        let sut = InternalAuthServiceImpl { jwt };

        // Act
        let result = sut.authenticate("token", "auth");

        // Assert
        result.unwrap();
    }

    #[test]
    fn invalid() {
        // Arrange
        let jwt = MockJwtService::new().with_verify(
            "token",
            Err(VerifyJwtError::<InternalAuthTokenData>::Invalid),
        );

        let sut = InternalAuthServiceImpl { jwt };

        // Act
        let result = sut.authenticate("token", "auth");

        // Assert
        assert_matches!(result, Err(InternalAuthError::InvalidToken));
    }

    #[test]
    fn expired() {
        // Arrange
        let jwt = MockJwtService::new().with_verify(
            "token",
            Err(VerifyJwtError::<InternalAuthTokenData>::Expired(
                InternalAuthTokenData { aud: "auth".into() },
            )),
        );

        let sut = InternalAuthServiceImpl { jwt };

        // Act
        let result = sut.authenticate("token", "auth");

        // Assert
        assert_matches!(result, Err(InternalAuthError::InvalidToken));
    }
}
