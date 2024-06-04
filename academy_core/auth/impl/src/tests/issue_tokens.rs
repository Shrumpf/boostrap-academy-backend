use academy_core_auth_contracts::{AuthService, Authentication, Tokens};
use academy_demo::{user::FOO, SHA256HASH1, UUID1};
use academy_shared_contracts::{
    hash::MockHashService, jwt::MockJwtService, secret::MockSecretService,
};

use crate::{tests::Sut, AuthServiceConfig, AuthServiceImpl, Token};

#[tokio::test]
async fn ok() {
    // Arrange
    let config = AuthServiceConfig::default();

    let expected = Tokens {
        access_token: "the access token jwt".into(),
        refresh_token: "some refresh token".into(),
        refresh_token_hash: (*SHA256HASH1).into(),
    };

    let auth = Authentication {
        user_id: FOO.user.id,
        session_id: UUID1.into(),
        refresh_token_hash: (*SHA256HASH1).into(),
        admin: FOO.user.admin,
        email_verified: FOO.user.email_verified,
    };

    let secret = MockSecretService::new()
        .with_generate(config.refresh_token_length, expected.refresh_token.clone());
    let hash = MockHashService::new().with_sha256(
        expected.refresh_token.clone().into_bytes(),
        *expected.refresh_token_hash,
    );
    let jwt = MockJwtService::new().with_sign(
        Token::from(auth),
        config.access_token_ttl,
        Ok(expected.access_token.clone()),
    );

    let sut = AuthServiceImpl {
        secret,
        hash,
        jwt,
        config,
        ..Sut::default()
    };

    // Act
    let result = sut.issue_tokens(&FOO.user, UUID1.into());

    // Assert
    assert_eq!(result.unwrap(), expected);
}
