use academy_core_user_contracts::{
    commands::verify_email::{MockUserVerifyEmailCommandService, UserVerifyEmailCommandError},
    UserService, UserVerifyEmailError,
};
use academy_demo::{user::FOO, VERIFICATION_CODE_1};
use academy_persistence_contracts::MockDatabase;
use academy_utils::assert_matches;

use crate::{tests::Sut, UserServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let db = MockDatabase::build(true);

    let user_verify_email = MockUserVerifyEmailCommandService::new()
        .with_invoke(VERIFICATION_CODE_1.clone(), Ok(FOO.clone()));

    let sut = UserServiceImpl {
        db,
        user_verify_email,
        ..Sut::default()
    };

    // Act
    let result = sut.verify_email(VERIFICATION_CODE_1.clone()).await;

    // Assert
    result.unwrap();
}

#[tokio::test]
async fn invalid_code() {
    // Arrange
    let db = MockDatabase::build(false);

    let user_verify_email = MockUserVerifyEmailCommandService::new().with_invoke(
        VERIFICATION_CODE_1.clone(),
        Err(UserVerifyEmailCommandError::InvalidCode),
    );

    let sut = UserServiceImpl {
        db,
        user_verify_email,
        ..Sut::default()
    };

    // Act
    let result = sut.verify_email(VERIFICATION_CODE_1.clone()).await;

    // Assert
    assert_matches!(result, Err(UserVerifyEmailError::InvalidCode));
}

#[tokio::test]
async fn already_verified() {
    // Arrange
    let db = MockDatabase::build(false);

    let user_verify_email = MockUserVerifyEmailCommandService::new().with_invoke(
        VERIFICATION_CODE_1.clone(),
        Err(UserVerifyEmailCommandError::AlreadyVerified),
    );

    let sut = UserServiceImpl {
        db,
        user_verify_email,
        ..Sut::default()
    };

    // Act
    let result = sut.verify_email(VERIFICATION_CODE_1.clone()).await;

    // Assert
    result.unwrap();
}
