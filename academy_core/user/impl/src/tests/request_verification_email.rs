use academy_auth_contracts::MockAuthService;
use academy_core_user_contracts::{
    commands::request_verification_email::MockUserRequestVerificationEmailCommandService,
    UserFeatureService, UserRequestVerificationEmailError,
};
use academy_demo::{
    session::{ADMIN_1, BAR_1, FOO_1},
    user::{ADMIN, BAR, FOO},
};
use academy_models::{
    auth::{AuthError, AuthenticateError, AuthorizeError},
    user::UserIdOrSelf,
};
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase};
use academy_utils::{assert_matches, Apply};

use crate::{tests::Sut, UserFeatureServiceImpl};

#[tokio::test]
async fn ok_self() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new().with_get_composite(
        FOO.user.id,
        Some(FOO.clone().with(|u| u.user.email_verified = false)),
    );

    let user_request_verification_email = MockUserRequestVerificationEmailCommandService::new()
        .with_invoke(
            FOO.user
                .email
                .clone()
                .unwrap()
                .with_name(FOO.profile.display_name.clone().into_inner()),
        );

    let sut = UserFeatureServiceImpl {
        auth,
        db,
        user_repo,
        user_request_verification_email,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_verification_email("token", UserIdOrSelf::Slf)
        .await;

    // Assert
    result.unwrap();
}

#[tokio::test]
async fn ok_admin() {
    // Arrange
    let auth =
        MockAuthService::new().with_authenticate(Some((ADMIN.user.clone(), ADMIN_1.clone())));

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new().with_get_composite(
        FOO.user.id,
        Some(FOO.clone().with(|u| u.user.email_verified = false)),
    );

    let user_request_verification_email = MockUserRequestVerificationEmailCommandService::new()
        .with_invoke(
            FOO.user
                .email
                .clone()
                .unwrap()
                .with_name(FOO.profile.display_name.clone().into_inner()),
        );

    let sut = UserFeatureServiceImpl {
        auth,
        db,
        user_repo,
        user_request_verification_email,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_verification_email("token", FOO.user.id.into())
        .await;

    // Assert
    result.unwrap();
}

#[tokio::test]
async fn unauthenticated() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(None);

    let sut = UserFeatureServiceImpl {
        auth,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_verification_email("token", FOO.user.id.into())
        .await;

    // Assert
    assert_matches!(
        result,
        Err(UserRequestVerificationEmailError::Auth(
            AuthError::Authenticate(AuthenticateError::InvalidToken)
        ))
    );
}

#[tokio::test]
async fn unauthorized() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((BAR.user.clone(), BAR_1.clone())));

    let sut = UserFeatureServiceImpl {
        auth,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_verification_email("token", FOO.user.id.into())
        .await;

    // Assert
    assert_matches!(
        result,
        Err(UserRequestVerificationEmailError::Auth(
            AuthError::Authorize(AuthorizeError::Admin)
        ))
    );
}

#[tokio::test]
async fn not_found() {
    // Arrange
    let auth =
        MockAuthService::new().with_authenticate(Some((ADMIN.user.clone(), ADMIN_1.clone())));

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new().with_get_composite(FOO.user.id, None);

    let sut = UserFeatureServiceImpl {
        auth,
        db,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_verification_email("token", FOO.user.id.into())
        .await;

    // Assert
    assert_matches!(result, Err(UserRequestVerificationEmailError::NotFound));
}

#[tokio::test]
async fn already_verified() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new().with_get_composite(FOO.user.id, Some(FOO.clone()));

    let sut = UserFeatureServiceImpl {
        auth,
        db,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_verification_email("token", UserIdOrSelf::Slf)
        .await;

    // Assert
    assert_matches!(
        result,
        Err(UserRequestVerificationEmailError::AlreadyVerified)
    );
}

#[tokio::test]
async fn no_email() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((BAR.user.clone(), BAR_1.clone())));

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new().with_get_composite(BAR.user.id, Some(BAR.clone()));

    let sut = UserFeatureServiceImpl {
        auth,
        db,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .request_verification_email("token", UserIdOrSelf::Slf)
        .await;

    // Assert
    assert_matches!(result, Err(UserRequestVerificationEmailError::NoEmail));
}
