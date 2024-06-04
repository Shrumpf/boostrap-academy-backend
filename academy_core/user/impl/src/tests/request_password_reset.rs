use academy_core_user_contracts::{
    commands::request_password_reset_email::MockUserRequestPasswordResetEmailCommandService,
    UserService,
};
use academy_demo::user::FOO;
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase};

use crate::{tests::Sut, UserServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), Some(FOO.clone()));

    let user_request_password_reset_email = MockUserRequestPasswordResetEmailCommandService::new()
        .with_invoke(FOO.user.id, FOO.user.email.clone().unwrap());

    let sut = UserServiceImpl {
        db,
        user_repo,
        user_request_password_reset_email,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_password_reset(FOO.user.email.clone().unwrap())
        .await;

    // Assert
    result.unwrap();
}

#[tokio::test]
async fn user_not_found() {
    // Arrange
    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), None);

    let sut = UserServiceImpl {
        db,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_password_reset(FOO.user.email.clone().unwrap())
        .await;

    // Assert
    result.unwrap();
}
