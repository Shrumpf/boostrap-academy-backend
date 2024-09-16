use academy_cache_contracts::MockCacheService;
use academy_core_session_contracts::session::MockSessionService;
use academy_core_user_contracts::{
    commands::create::{MockUserCreateCommandService, UserCreateCommand, UserCreateCommandError},
    UserCreateError, UserCreateRequest, UserFeatureService,
};
use academy_demo::{
    oauth2::{FOO_OAUTH2_LINK_1, TEST_OAUTH2_PROVIDER_ID},
    session::FOO_1,
    user::FOO,
};
use academy_models::{auth::Login, oauth2::OAuth2Registration};
use academy_persistence_contracts::MockDatabase;
use academy_shared_contracts::captcha::{CaptchaCheckError, MockCaptchaService};
use academy_utils::assert_matches;

use crate::{tests::Sut, UserFeatureServiceImpl};

#[tokio::test]
async fn ok() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: Some("secure password".try_into().unwrap()),
        oauth2_registration_token: None,
    };

    let expected = Login {
        user_composite: FOO.clone(),
        session: FOO_1.clone(),
        access_token: "the access token".into(),
        refresh_token: "some refresh token".into(),
    };

    let db = MockDatabase::build(true);

    let captcha = MockCaptchaService::new().with_check(Some("resp"), Ok(()));

    let user_create =
        MockUserCreateCommandService::new().with_invoke(req_to_cmd(&request), Ok(FOO.clone()));

    let session = MockSessionService::new().with_create(
        FOO.clone(),
        FOO_1.device_name.clone(),
        true,
        expected.clone(),
    );

    let sut = UserFeatureServiceImpl {
        db,
        captcha,
        user_create,
        session,
        ..Sut::default()
    };

    // Act
    let result = sut
        .create_user(
            request,
            FOO_1.device_name.clone(),
            Some("resp".try_into().unwrap()),
        )
        .await;

    // Assert
    assert_eq!(result.unwrap(), expected);
}

#[tokio::test]
async fn ok_oauth2() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: None,
        oauth2_registration_token: Some(
            "K7oACiokVoyttnGgYxJwCc2VCvDbQI10Bewthc5exlyQly2JZCViycDereak92oB"
                .try_into()
                .unwrap(),
        ),
    };

    let expected = Login {
        user_composite: FOO.clone(),
        session: FOO_1.clone(),
        access_token: "the access token".into(),
        refresh_token: "some refresh token".into(),
    };

    let db = MockDatabase::build(true);

    let captcha = MockCaptchaService::new().with_check(Some("resp"), Ok(()));

    let cache_key =
        "oauth2_registration:K7oACiokVoyttnGgYxJwCc2VCvDbQI10Bewthc5exlyQly2JZCViycDereak92oB";
    let cache = MockCacheService::new()
        .with_get(
            cache_key.into(),
            Some(OAuth2Registration {
                provider_id: TEST_OAUTH2_PROVIDER_ID.clone(),
                remote_user: FOO_OAUTH2_LINK_1.remote_user.clone(),
            }),
        )
        .with_remove(cache_key.into());

    let user_create =
        MockUserCreateCommandService::new().with_invoke(req_to_cmd(&request), Ok(FOO.clone()));

    let session = MockSessionService::new().with_create(
        FOO.clone(),
        FOO_1.device_name.clone(),
        true,
        expected.clone(),
    );

    let sut = UserFeatureServiceImpl {
        db,
        cache,
        captcha,
        user_create,
        session,
        ..Sut::default()
    };

    // Act
    let result = sut
        .create_user(
            request,
            FOO_1.device_name.clone(),
            Some("resp".try_into().unwrap()),
        )
        .await;

    // Assert
    assert_eq!(result.unwrap(), expected);
}

#[tokio::test]
async fn no_login_method() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: None,
        oauth2_registration_token: None,
    };

    let sut = Sut::default();

    // Act
    let result = sut
        .create_user(request, FOO_1.device_name.clone(), None)
        .await;

    // Assert
    assert_matches!(result, Err(UserCreateError::NoLoginMethod));
}

#[tokio::test]
async fn invalid_recaptcha_response() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: Some("secure password".try_into().unwrap()),
        oauth2_registration_token: None,
    };

    let captcha =
        MockCaptchaService::new().with_check(Some("resp"), Err(CaptchaCheckError::Failed));

    let sut = UserFeatureServiceImpl {
        captcha,
        ..Sut::default()
    };

    // Act
    let result = sut
        .create_user(
            request,
            FOO_1.device_name.clone(),
            Some("resp".try_into().unwrap()),
        )
        .await;

    // Assert
    assert_matches!(result, Err(UserCreateError::Recaptcha));
}

#[tokio::test]
async fn name_conflict() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: Some("secure password".try_into().unwrap()),
        oauth2_registration_token: None,
    };

    let db = MockDatabase::build(false);

    let captcha = MockCaptchaService::new().with_check(None, Ok(()));

    let user_create = MockUserCreateCommandService::new().with_invoke(
        req_to_cmd(&request),
        Err(UserCreateCommandError::NameConflict),
    );

    let sut = UserFeatureServiceImpl {
        db,
        captcha,
        user_create,
        ..Sut::default()
    };

    // Act
    let result = sut
        .create_user(request, FOO_1.device_name.clone(), None)
        .await;

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
        password: Some("secure password".try_into().unwrap()),
        oauth2_registration_token: None,
    };

    let db = MockDatabase::build(false);

    let captcha = MockCaptchaService::new().with_check(None, Ok(()));

    let user_create = MockUserCreateCommandService::new().with_invoke(
        req_to_cmd(&request),
        Err(UserCreateCommandError::EmailConflict),
    );

    let sut = UserFeatureServiceImpl {
        db,
        captcha,
        user_create,
        ..Sut::default()
    };

    // Act
    let result = sut
        .create_user(request, FOO_1.device_name.clone(), None)
        .await;

    // Assert
    assert_matches!(result, Err(UserCreateError::EmailConflict));
}

#[tokio::test]
async fn oauth2_invalid_registration_token() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: None,
        oauth2_registration_token: Some(
            "K7oACiokVoyttnGgYxJwCc2VCvDbQI10Bewthc5exlyQly2JZCViycDereak92oB"
                .try_into()
                .unwrap(),
        ),
    };

    let captcha = MockCaptchaService::new().with_check(Some("resp"), Ok(()));

    let cache = MockCacheService::new().with_get(
        "oauth2_registration:K7oACiokVoyttnGgYxJwCc2VCvDbQI10Bewthc5exlyQly2JZCViycDereak92oB"
            .into(),
        None::<OAuth2Registration>,
    );

    let sut = UserFeatureServiceImpl {
        cache,
        captcha,
        ..Sut::default()
    };

    // Act
    let result = sut
        .create_user(
            request,
            FOO_1.device_name.clone(),
            Some("resp".try_into().unwrap()),
        )
        .await;

    // Assert
    assert_matches!(result, Err(UserCreateError::InvalidOAuthRegistrationToken));
}

#[tokio::test]
async fn oauth2_remote_already_linked() {
    // Arrange
    let request = UserCreateRequest {
        name: FOO.user.name.clone(),
        display_name: FOO.profile.display_name.clone(),
        email: FOO.user.email.clone().unwrap(),
        password: None,
        oauth2_registration_token: Some(
            "K7oACiokVoyttnGgYxJwCc2VCvDbQI10Bewthc5exlyQly2JZCViycDereak92oB"
                .try_into()
                .unwrap(),
        ),
    };

    let db = MockDatabase::build(false);

    let captcha = MockCaptchaService::new().with_check(Some("resp"), Ok(()));

    let cache_key =
        "oauth2_registration:K7oACiokVoyttnGgYxJwCc2VCvDbQI10Bewthc5exlyQly2JZCViycDereak92oB";
    let cache = MockCacheService::new().with_get(
        cache_key.into(),
        Some(OAuth2Registration {
            provider_id: TEST_OAUTH2_PROVIDER_ID.clone(),
            remote_user: FOO_OAUTH2_LINK_1.remote_user.clone(),
        }),
    );

    let user_create = MockUserCreateCommandService::new().with_invoke(
        req_to_cmd(&request),
        Err(UserCreateCommandError::RemoteAlreadyLinked),
    );

    let sut = UserFeatureServiceImpl {
        db,
        cache,
        captcha,
        user_create,
        ..Sut::default()
    };

    // Act
    let result = sut
        .create_user(
            request,
            FOO_1.device_name.clone(),
            Some("resp".try_into().unwrap()),
        )
        .await;

    // Assert
    assert_matches!(result, Err(UserCreateError::RemoteAlreadyLinked));
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
        oauth2_registration: req
            .oauth2_registration_token
            .as_ref()
            .map(|_| OAuth2Registration {
                provider_id: TEST_OAUTH2_PROVIDER_ID.clone(),
                remote_user: FOO_OAUTH2_LINK_1.remote_user.clone(),
            }),
    }
}
