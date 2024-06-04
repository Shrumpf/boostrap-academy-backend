use academy_cache_contracts::MockCacheService;
use academy_core_auth_contracts::{AuthService, Authentication};
use academy_demo::{user::FOO, SHA256HASH1, SHA256HASH1_HEX, UUID1};
use academy_models::auth::AuthenticateError;
use academy_shared_contracts::jwt::{MockJwtService, VerifyJwtError};
use academy_utils::assert_matches;

use crate::{tests::Sut, AuthServiceImpl, Token};

#[tokio::test]
async fn ok() {
    // Arrange
    let expected = Authentication {
        user_id: FOO.user.id,
        session_id: UUID1.into(),
        refresh_token_hash: (*SHA256HASH1).into(),
        admin: FOO.user.admin,
        email_verified: FOO.user.email_verified,
    };

    let jwt = MockJwtService::new().with_verify("my auth token", Ok(Token::from(expected)));

    let cache = MockCacheService::new().with_get(
        format!("access_token_invalidated:{SHA256HASH1_HEX}"),
        None::<()>,
    );

    let sut = AuthServiceImpl {
        jwt,
        cache,
        ..Sut::default()
    };

    // Act
    let result = sut.authenticate("my auth token").await;

    // Assert
    assert_eq!(result.unwrap(), expected);
}

#[tokio::test]
async fn invalid_token() {
    // Arrange
    let jwt =
        MockJwtService::new().with_verify("my auth token", Err(VerifyJwtError::<Token>::Invalid));

    let sut = AuthServiceImpl {
        jwt,
        ..Sut::default()
    };

    // Act
    let result = sut.authenticate("my auth token").await;

    // Assert
    assert_matches!(result, Err(AuthenticateError::InvalidToken));
}

#[tokio::test]
async fn expired() {
    // Arrange
    let expected = Authentication {
        user_id: FOO.user.id,
        session_id: UUID1.into(),
        refresh_token_hash: (*SHA256HASH1).into(),
        admin: FOO.user.admin,
        email_verified: FOO.user.email_verified,
    };

    let jwt = MockJwtService::new().with_verify(
        "my auth token",
        Err(VerifyJwtError::Expired(Token::from(expected))),
    );

    let sut = AuthServiceImpl {
        jwt,
        ..Sut::default()
    };

    // Act
    let result = sut.authenticate("my auth token").await;

    // Assert
    assert_matches!(result, Err(AuthenticateError::InvalidToken));
}

#[tokio::test]
async fn access_token_revoked() {
    // Arrange
    let expected = Authentication {
        user_id: FOO.user.id,
        session_id: UUID1.into(),
        refresh_token_hash: (*SHA256HASH1).into(),
        admin: FOO.user.admin,
        email_verified: FOO.user.email_verified,
    };

    let jwt = MockJwtService::new().with_verify("my auth token", Ok(Token::from(expected)));

    let cache = MockCacheService::new().with_get(
        format!("access_token_invalidated:{SHA256HASH1_HEX}"),
        Some(()),
    );

    let sut = AuthServiceImpl {
        jwt,
        cache,
        ..Sut::default()
    };

    // Act
    let result = sut.authenticate("my auth token").await;

    // Assert
    assert_matches!(result, Err(AuthenticateError::InvalidToken));
}
