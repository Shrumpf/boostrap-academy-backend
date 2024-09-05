use academy_core_internal_contracts::{
    auth::{InternalAuthError, MockInternalAuthService},
    InternalGetUserByEmailError, InternalService,
};
use academy_demo::user::FOO;
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase};
use academy_utils::assert_matches;

use crate::{tests::Sut, InternalServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let internal_auth = MockInternalAuthService::new().with_authenticate("auth", true);

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), Some(FOO.clone()));

    let sut = InternalServiceImpl {
        db,
        internal_auth,
        user_repo,
    };

    // Act
    let result = sut
        .get_user_by_email("token", FOO.user.email.clone().unwrap())
        .await;

    // Assert
    assert_eq!(result.unwrap(), *FOO);
}

#[tokio::test]
async fn unauthenticated() {
    // Arrange
    let internal_auth = MockInternalAuthService::new().with_authenticate("auth", false);

    let sut = InternalServiceImpl {
        internal_auth,
        ..Sut::default()
    };

    // Act
    let result = sut
        .get_user_by_email("token", FOO.user.email.clone().unwrap())
        .await;

    // Assert
    assert_matches!(
        result,
        Err(InternalGetUserByEmailError::Auth(
            InternalAuthError::InvalidToken
        ))
    );
}

#[tokio::test]
async fn not_found() {
    // Arrange
    let internal_auth = MockInternalAuthService::new().with_authenticate("auth", true);

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), None);

    let sut = InternalServiceImpl {
        db,
        internal_auth,
        user_repo,
    };

    // Act
    let result = sut
        .get_user_by_email("token", FOO.user.email.clone().unwrap())
        .await;

    // Assert
    assert_matches!(result, Err(InternalGetUserByEmailError::NotFound));
}
