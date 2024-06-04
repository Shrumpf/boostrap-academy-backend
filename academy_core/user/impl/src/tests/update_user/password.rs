use academy_core_auth_contracts::MockAuthService;
use academy_core_user_contracts::{
    commands::update_password::MockUserUpdatePasswordCommandService, PasswordUpdate, UserService,
    UserUpdateError, UserUpdateRequest, UserUpdateUserRequest,
};
use academy_demo::{session::FOO_1, user::FOO};
use academy_models::user::{UserIdOrSelf, UserPassword};
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase};
use academy_utils::{assert_matches, patch::PatchValue};

use crate::{tests::Sut, UserServiceImpl};

#[tokio::test]
async fn update_password() {
    // Arrange
    let new_password = UserPassword::try_new("the new password").unwrap();

    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let db = MockDatabase::build(true);

    let user_repo = MockUserRepository::new().with_get_composite(FOO.user.id, Some(FOO.clone()));

    let user_update_password =
        MockUserUpdatePasswordCommandService::new().with_invoke(FOO.user.id, new_password.clone());

    let sut = UserServiceImpl {
        auth,
        db,
        user_update_password,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .update_user(
            "token",
            UserIdOrSelf::Slf,
            UserUpdateRequest {
                user: UserUpdateUserRequest {
                    password: PatchValue::Update(PasswordUpdate::Change(new_password)),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;

    // Assert
    assert_eq!(result.unwrap(), *FOO);
}

#[tokio::test]
async fn remove_password() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new().with_get_composite(FOO.user.id, Some(FOO.clone()));

    let sut = UserServiceImpl {
        auth,
        db,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .update_user(
            "token",
            UserIdOrSelf::Slf,
            UserUpdateRequest {
                user: UserUpdateUserRequest {
                    password: PatchValue::Update(PasswordUpdate::Remove),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;

    // Assert
    assert_matches!(result, Err(UserUpdateError::CannotRemovePassword));
}
