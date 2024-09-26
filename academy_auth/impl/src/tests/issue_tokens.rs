use academy_auth_contracts::{
    access_token::MockAuthAccessTokenService, refresh_token::MockAuthRefreshTokenService,
    AuthService, Tokens,
};
use academy_demo::{user::FOO, SHA256HASH1, UUID1};

use crate::{tests::Sut, AuthServiceConfig, AuthServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let config = AuthServiceConfig::default();

    let expected = Tokens {
        access_token: "the access token jwt".into(),
        refresh_token: "some refresh token".into(),
        refresh_token_hash: (*SHA256HASH1).into(),
    };

    let auth_access_token = MockAuthAccessTokenService::new().with_issue(
        FOO.user.clone(),
        UUID1.into(),
        (*SHA256HASH1).into(),
        expected.access_token.clone(),
    );

    let auth_refresh_token = MockAuthRefreshTokenService::new()
        .with_issue(expected.refresh_token.clone())
        .with_hash(expected.refresh_token.clone(), expected.refresh_token_hash);

    let sut = AuthServiceImpl {
        config,
        auth_access_token,
        auth_refresh_token,
        ..Sut::default()
    };

    // Act
    let result = sut.issue_tokens(&FOO.user, UUID1.into());

    // Assert
    assert_eq!(result.unwrap(), expected);
}
