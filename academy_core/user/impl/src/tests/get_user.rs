use academy_auth_contracts::MockAuthService;
use academy_core_user_contracts::{UserGetError, UserService};
use academy_demo::{
    session::{ADMIN_1, BAR_1, FOO_1},
    user::{ADMIN, BAR, FOO},
};
use academy_models::{
    auth::{AuthError, AuthenticateError, AuthorizeError},
    user::UserIdOrSelf,
};
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase};
use academy_utils::assert_matches;

use crate::{tests::Sut, UserServiceImpl};

#[tokio::test]
async fn ok_self() {
    // Arrange
    let db = MockDatabase::build(false);
    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let user_repo = MockUserRepository::new().with_get_composite(FOO.user.id, Some(FOO.clone()));

    let sut = UserServiceImpl {
        db,
        auth,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut.get_user("token", UserIdOrSelf::Slf).await;

    // Assert
    assert_eq!(result.unwrap(), *FOO);
}

#[tokio::test]
async fn ok_admin() {
    // Arrange
    let db = MockDatabase::build(false);
    let auth =
        MockAuthService::new().with_authenticate(Some((ADMIN.user.clone(), ADMIN_1.clone())));

    let user_repo = MockUserRepository::new().with_get_composite(FOO.user.id, Some(FOO.clone()));

    let sut = UserServiceImpl {
        db,
        auth,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut.get_user("token", FOO.user.id.into()).await;

    // Assert
    assert_eq!(result.unwrap(), *FOO);
}

#[tokio::test]
async fn unauthenticated() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(None);

    let sut = UserServiceImpl {
        auth,
        ..Sut::default()
    };

    // Act
    let result = sut.get_user("token", FOO.user.id.into()).await;

    // Assert
    assert_matches!(
        result,
        Err(UserGetError::Auth(AuthError::Authenticate(
            AuthenticateError::InvalidToken
        )))
    );
}

#[tokio::test]
async fn unauthorized() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((BAR.user.clone(), BAR_1.clone())));

    let sut = UserServiceImpl {
        auth,
        ..Sut::default()
    };

    // Act
    let result = sut.get_user("token", FOO.user.id.into()).await;

    // Assert
    assert_matches!(
        result,
        Err(UserGetError::Auth(AuthError::Authorize(
            AuthorizeError::Admin
        )))
    );
}

#[tokio::test]
async fn not_found() {
    // Arrange
    let db = MockDatabase::build(false);
    let auth =
        MockAuthService::new().with_authenticate(Some((ADMIN.user.clone(), ADMIN_1.clone())));

    let user_repo = MockUserRepository::new().with_get_composite(FOO.user.id, None);

    let sut = UserServiceImpl {
        db,
        auth,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut.get_user("token", FOO.user.id.into()).await;

    // Assert
    assert_matches!(result, Err(UserGetError::NotFound));
}
