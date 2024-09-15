use academy_core_user_contracts::{
    commands::reset_password::{
        MockUserResetPasswordCommandService, UserResetPasswordCommandError,
    },
    UserFeatureService, UserResetPasswordError,
};
use academy_demo::{
    user::{FOO, FOO_PASSWORD},
    VERIFICATION_CODE_1,
};
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase};
use academy_utils::assert_matches;

use crate::{tests::Sut, UserFeatureServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let db = MockDatabase::build(true);

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), Some(FOO.clone()));

    let user_reset_password = MockUserResetPasswordCommandService::new().with_invoke(
        FOO.user.id,
        VERIFICATION_CODE_1.clone(),
        FOO_PASSWORD.clone(),
        Ok(()),
    );

    let sut = UserFeatureServiceImpl {
        db,
        user_repo,
        user_reset_password,
        ..Sut::default()
    };

    // Act
    let result = sut
        .reset_password(
            FOO.user.email.clone().unwrap(),
            VERIFICATION_CODE_1.clone(),
            FOO_PASSWORD.clone(),
        )
        .await;

    // Act
    assert_eq!(result.unwrap(), *FOO);
}

#[tokio::test]
async fn user_not_found() {
    // Arrange
    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), None);

    let sut = UserFeatureServiceImpl {
        db,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .reset_password(
            FOO.user.email.clone().unwrap(),
            VERIFICATION_CODE_1.clone(),
            FOO_PASSWORD.clone(),
        )
        .await;

    // Act
    assert_matches!(result, Err(UserResetPasswordError::Failed));
}

#[tokio::test]
async fn invalid_code() {
    // Arrange
    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new()
        .with_get_composite_by_email(FOO.user.email.clone().unwrap(), Some(FOO.clone()));

    let user_reset_password = MockUserResetPasswordCommandService::new().with_invoke(
        FOO.user.id,
        VERIFICATION_CODE_1.clone(),
        FOO_PASSWORD.clone(),
        Err(UserResetPasswordCommandError::InvalidCode),
    );

    let sut = UserFeatureServiceImpl {
        db,
        user_repo,
        user_reset_password,
        ..Sut::default()
    };

    // Act
    let result = sut
        .reset_password(
            FOO.user.email.clone().unwrap(),
            VERIFICATION_CODE_1.clone(),
            FOO_PASSWORD.clone(),
        )
        .await;

    // Act
    assert_matches!(result, Err(UserResetPasswordError::Failed));
}
