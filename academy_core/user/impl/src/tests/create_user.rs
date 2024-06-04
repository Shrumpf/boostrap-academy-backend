use academy_core_session_contracts::commands::create::MockSessionCreateCommandService;
use academy_core_user_contracts::{
    commands::create::{MockUserCreateCommandService, UserCreateCommand, UserCreateCommandError},
    UserCreateError, UserCreateRequest, UserService,
};
use academy_demo::{session::FOO_1, user::FOO};
use academy_models::auth::Login;
use academy_persistence_contracts::MockDatabase;
use academy_utils::assert_matches;

use crate::{tests::Sut, UserServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: "secure password".try_into().unwrap(),
    };

    let expected = Login {
        user_composite: FOO.clone(),
        session: FOO_1.clone(),
        access_token: "the access token".into(),
        refresh_token: "some refresh token".into(),
    };

    let db = MockDatabase::build(true);

    let user_create =
        MockUserCreateCommandService::new().with_invoke(req_to_cmd(&request), Ok(FOO.clone()));

    let session_create = MockSessionCreateCommandService::new().with_invoke(
        FOO.clone(),
        FOO_1.device_name.clone(),
        true,
        expected.clone(),
    );

    let sut = UserServiceImpl {
        db,
        user_create,
        session_create,
        ..Sut::default()
    };

    // Act
    let result = sut.create_user(request, FOO_1.device_name.clone()).await;

    // Assert
    assert_eq!(result.unwrap(), expected);
}

#[tokio::test]
async fn name_conflict() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: "secure password".try_into().unwrap(),
    };

    let db = MockDatabase::build(false);

    let user_create = MockUserCreateCommandService::new().with_invoke(
        req_to_cmd(&request),
        Err(UserCreateCommandError::NameConflict),
    );

    let sut = UserServiceImpl {
        db,
        user_create,
        ..Sut::default()
    };

    // Act
    let result = sut.create_user(request, FOO_1.device_name.clone()).await;

    // Assert
    assert_matches!(result, Err(UserCreateError::NameConflict));
}

#[tokio::test]
async fn email_conflict() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: "secure password".try_into().unwrap(),
    };

    let db = MockDatabase::build(false);

    let user_create = MockUserCreateCommandService::new().with_invoke(
        req_to_cmd(&request),
        Err(UserCreateCommandError::EmailConflict),
    );

    let sut = UserServiceImpl {
        db,
        user_create,
        ..Sut::default()
    };

    // Act
    let result = sut.create_user(request, FOO_1.device_name.clone()).await;

    // Assert
    assert_matches!(result, Err(UserCreateError::EmailConflict));
}

fn req_to_cmd(req: &UserCreateRequest) -> UserCreateCommand {
    UserCreateCommand {
        name: req.name.clone(),
        display_name: req.display_name.clone(),
        email: req.email.clone(),
        password: req.password.clone(),
        admin: false,
        enabled: true,
        email_verified: false,
    }
}
