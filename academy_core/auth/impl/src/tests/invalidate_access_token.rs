use academy_core_auth_contracts::{
    commands::invalidate_access_token::MockAuthInvalidateAccessTokenCommandService, AuthService,
};
use academy_demo::SHA256HASH1;
use academy_models::session::SessionRefreshTokenHash;

use crate::{tests::Sut, AuthServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let rth = SessionRefreshTokenHash::from(*SHA256HASH1);

    let auth_invalidate_access_token =
        MockAuthInvalidateAccessTokenCommandService::new().with_invoke(rth);

    let sut = AuthServiceImpl {
        auth_invalidate_access_token,
        ..Sut::default()
    };

    // Act
    let result = sut.invalidate_access_token(rth).await;

    // Assert
    result.unwrap();
}
