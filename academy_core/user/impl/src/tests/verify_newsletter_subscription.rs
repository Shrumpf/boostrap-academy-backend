use academy_core_auth_contracts::MockAuthService;
use academy_core_user_contracts::{
    commands::verify_newsletter_subscription::{
        MockUserVerifyNewsletterSubscriptionCommandService,
        UserVerifyNewsletterSubscriptionCommandError,
    },
    UserService, UserVerifyNewsletterSubscriptionError,
};
use academy_demo::{
    session::{BAR_1, FOO_1},
    user::{BAR, FOO},
    VERIFICATION_CODE_1,
};
use academy_models::{
    auth::{AuthError, AuthenticateError, AuthorizeError},
    user::UserIdOrSelf,
};
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase};
use academy_utils::{assert_matches, Apply};

use crate::{tests::Sut, UserServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let db = MockDatabase::build(true);

    let user_repo = MockUserRepository::new().with_get_composite(
        FOO.user.id,
        Some(FOO.clone().with(|u| u.user.newsletter = false)),
    );

    let user_verify_newsletter_subscription =
        MockUserVerifyNewsletterSubscriptionCommandService::new().with_invoke(
            FOO.user.id,
            VERIFICATION_CODE_1.clone(),
            Ok(()),
        );

    let sut = UserServiceImpl {
        auth,
        db,
        user_verify_newsletter_subscription,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .verify_newsletter_subscription("token", UserIdOrSelf::Slf, VERIFICATION_CODE_1.clone())
        .await;

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
    let result = sut
        .verify_newsletter_subscription("token", FOO.user.id.into(), VERIFICATION_CODE_1.clone())
        .await;

    // Assert
    assert_matches!(
        result,
        Err(UserVerifyNewsletterSubscriptionError::Auth(
            AuthError::Authenticate(AuthenticateError::InvalidToken)
        ))
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
    let result = sut
        .verify_newsletter_subscription("token", FOO.user.id.into(), VERIFICATION_CODE_1.clone())
        .await;

    // Assert
    assert_matches!(
        result,
        Err(UserVerifyNewsletterSubscriptionError::Auth(
            AuthError::Authorize(AuthorizeError::Admin)
        ))
    );
}

#[tokio::test]
async fn not_found() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new().with_get_composite(FOO.user.id, None);

    let sut = UserServiceImpl {
        auth,
        db,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .verify_newsletter_subscription("token", UserIdOrSelf::Slf, VERIFICATION_CODE_1.clone())
        .await;

    // Assert
    assert_matches!(result, Err(UserVerifyNewsletterSubscriptionError::NotFound));
}

#[tokio::test]
async fn already_subscribed() {
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
        .verify_newsletter_subscription("token", UserIdOrSelf::Slf, VERIFICATION_CODE_1.clone())
        .await;

    // Assert
    assert_matches!(
        result,
        Err(UserVerifyNewsletterSubscriptionError::AlreadySubscribed)
    );
}

#[tokio::test]
async fn invalid_code() {
    // Arrange
    let auth = MockAuthService::new().with_authenticate(Some((FOO.user.clone(), FOO_1.clone())));

    let db = MockDatabase::build(false);

    let user_repo = MockUserRepository::new().with_get_composite(
        FOO.user.id,
        Some(FOO.clone().with(|u| u.user.newsletter = false)),
    );

    let user_verify_newsletter_subscription =
        MockUserVerifyNewsletterSubscriptionCommandService::new().with_invoke(
            FOO.user.id,
            VERIFICATION_CODE_1.clone(),
            Err(UserVerifyNewsletterSubscriptionCommandError::InvalidCode),
        );

    let sut = UserServiceImpl {
        auth,
        db,
        user_verify_newsletter_subscription,
        user_repo,
        ..Sut::default()
    };

    // Act
    let result = sut
        .verify_newsletter_subscription("token", UserIdOrSelf::Slf, VERIFICATION_CODE_1.clone())
        .await;

    // Assert
    assert_matches!(
        result,
        Err(UserVerifyNewsletterSubscriptionError::InvalidCode)
    );
}
