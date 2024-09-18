use academy_core_user_contracts::{
    email_confirmation::{MockUserEmailConfirmationService, UserEmailConfirmationVerifyEmailError},
    UserFeatureService, UserVerifyEmailError,
};
use academy_demo::{user::FOO, VERIFICATION_CODE_1};
use academy_persistence_contracts::MockDatabase;
use academy_utils::assert_matches;

use crate::{tests::Sut, UserFeatureServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let db = MockDatabase::build(true);

    let user_email_confirmation = MockUserEmailConfirmationService::new()
        .with_verify_email(VERIFICATION_CODE_1.clone(), Ok(FOO.clone()));

    let sut = UserFeatureServiceImpl {
        db,
        user_email_confirmation,
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

    let user_email_confirmation = MockUserEmailConfirmationService::new().with_verify_email(
        VERIFICATION_CODE_1.clone(),
        Err(UserEmailConfirmationVerifyEmailError::InvalidCode),
    );

    let sut = UserFeatureServiceImpl {
        db,
        user_email_confirmation,
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

    let user_email_confirmation = MockUserEmailConfirmationService::new().with_verify_email(
        VERIFICATION_CODE_1.clone(),
        Err(UserEmailConfirmationVerifyEmailError::AlreadyVerified),
    );

    let sut = UserFeatureServiceImpl {
        db,
        user_email_confirmation,
        ..Sut::default()
    };

    // Act
    let result = sut.verify_email(VERIFICATION_CODE_1.clone()).await;

    // Assert
    result.unwrap();
}
