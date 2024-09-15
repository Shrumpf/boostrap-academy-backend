use academy_auth_contracts::MockAuthService;
use academy_core_session_contracts::{
    commands::delete::MockSessionDeleteCommandService, SessionDeleteCurrentError, SessionService,
};
use academy_demo::{session::FOO_1, user::FOO};
use academy_models::auth::{AuthError, AuthenticateError};
use academy_persistence_contracts::MockDatabase;
use academy_utils::assert_matches;

use crate::{tests::Sut, SessionServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let db = MockDatabase::build(true);

    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let session_delete = MockSessionDeleteCommandService::new().with_invoke(FOO_1.id, true);

    let sut = SessionServiceImpl {
        db,
        auth,
        session_delete,
        ..Sut::default()
    };

    // Act
    let result = sut.delete_current_session("token").await;

    // Assert
    result.unwrap();
}

#[tokio::test]
async fn unauthenticated() {
    // Arrange

    let auth = MockAuthService::new().with_authenticate(None);

    let sut = SessionServiceImpl {
        auth,
        ..Sut::default()
    };

    // Act
    let result = sut.delete_current_session("token").await;

    // Assert
    assert_matches!(
        result,
        Err(SessionDeleteCurrentError::Auth(AuthError::Authenticate(
            AuthenticateError::InvalidToken
        )))
    );
}
