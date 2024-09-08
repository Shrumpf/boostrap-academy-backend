use std::sync::Arc;

use academy_core_user_contracts::{
    queries::list::{UserListQuery, UserListResult},
    PasswordUpdate, UserCreateError, UserCreateRequest, UserDeleteError, UserGetError,
    UserListError, UserRequestPasswordResetError, UserRequestVerificationEmailError,
    UserResetPasswordError, UserService, UserUpdateError, UserUpdateRequest, UserUpdateUserRequest,
    UserVerifyEmailError, UserVerifyNewsletterSubscriptionError,
};
use academy_models::{
    email_address::EmailAddress,
    oauth2::OAuth2RegistrationToken,
    session::DeviceName,
    user::{
        UserBio, UserCity, UserCountry, UserDisplayName, UserFirstName, UserInvoiceInfo,
        UserLastName, UserName, UserPassword, UserProfilePatch, UserStreet, UserTags, UserVatId,
        UserZipCode,
    },
    RecaptchaResponse, VerificationCode,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use serde::{Deserialize, Serialize};

use super::{auth_error, error, internal_server_error};
use crate::{
    extractors::{auth::ApiToken, user_agent::UserAgent},
    models::{
        session::ApiLogin,
        user::{ApiUser, ApiUserFilter, ApiUserIdOrSelf, ApiUserPasswordOrEmpty},
        ApiPaginationSlice,
    },
};

pub fn router(service: Arc<impl UserService>) -> Router<()> {
    Router::new()
        .route("/auth/users", routing::get(list).post(create))
        .route(
            "/auth/users/:user_id",
            routing::get(get).patch(update).delete(delete),
        )
        .route(
            "/auth/users/:user_id/email",
            routing::post(request_verification_email).put(verify_email),
        )
        .route(
            "/auth/users/:user_id/newsletter",
            routing::put(verify_newsletter_subscription),
        )
        .route(
            "/auth/password_reset",
            routing::post(request_password_reset).put(reset_password),
        )
        .with_state(service)
}

#[derive(Serialize)]
struct ListResult {
    total: u64,
    users: Vec<ApiUser>,
}

async fn list(
    user_service: State<Arc<impl UserService>>,
    token: ApiToken,
    Query(pagination): Query<ApiPaginationSlice>,
    Query(filter): Query<ApiUserFilter>,
) -> Response {
    match user_service
        .list_users(
            &token.0,
            UserListQuery {
                filter: filter.into(),
                pagination: pagination.into(),
            },
        )
        .await
    {
        Ok(UserListResult {
            total,
            user_composites: users,
        }) => Json(ListResult {
            total,
            users: users.into_iter().map(Into::into).collect(),
        })
        .into_response(),
        Err(UserListError::Auth(err)) => auth_error(err),
        Err(UserListError::Other(err)) => internal_server_error(err),
    }
}

async fn get(
    user_service: State<Arc<impl UserService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
) -> Response {
    match user_service.get_user(&token.0, user_id.into()).await {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(UserGetError::Auth(err)) => auth_error(err),
        Err(UserGetError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(UserGetError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct CreateRequest {
    name: UserName,
    display_name: UserDisplayName,
    email: EmailAddress,
    password: Option<UserPassword>,
    oauth_register_token: Option<OAuth2RegistrationToken>,
    recaptcha_response: Option<RecaptchaResponse>,
}

async fn create(
    user_service: State<Arc<impl UserService>>,
    user_agent: UserAgent,
    Json(CreateRequest {
        name,
        display_name,
        email,
        password,
        oauth_register_token,
        recaptcha_response,
    }): Json<CreateRequest>,
) -> Response {
    match user_service
        .create_user(
            UserCreateRequest {
                name,
                display_name,
                email,
                password,
                oauth2_registration_token: oauth_register_token,
            },
            user_agent.0.map(DeviceName::from_string_truncated),
            recaptcha_response,
        )
        .await
    {
        Ok(result) => (StatusCode::CREATED, Json(ApiLogin::from(result))).into_response(),
        Err(UserCreateError::NameConflict) => error(StatusCode::CONFLICT, "User already exists"),
        Err(UserCreateError::EmailConflict) => error(StatusCode::CONFLICT, "Email already exists"),
        Err(UserCreateError::Recaptcha) => {
            error(StatusCode::PRECONDITION_FAILED, "Recaptcha failed")
        }
        Err(UserCreateError::NoLoginMethod) => {
            error(StatusCode::PRECONDITION_FAILED, "No login method")
        }
        Err(UserCreateError::InvalidOAuthRegistrationToken) => {
            error(StatusCode::UNAUTHORIZED, "Invalid OAuth token")
        }
        Err(UserCreateError::RemoteAlreadyLinked) => {
            error(StatusCode::CONFLICT, "Remote already linked")
        }
        Err(UserCreateError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct UpdateRequest {
    name: Option<UserName>,
    display_name: Option<UserDisplayName>,
    email: Option<EmailAddress>,
    email_verified: Option<bool>,
    password: Option<ApiUserPasswordOrEmpty>,
    enabled: Option<bool>,
    admin: Option<bool>,
    description: Option<UserBio>,
    tags: Option<UserTags>,
    newsletter: Option<bool>,
    business: Option<bool>,
    first_name: Option<UserFirstName>,
    last_name: Option<UserLastName>,
    street: Option<UserStreet>,
    zip_code: Option<UserZipCode>,
    city: Option<UserCity>,
    country: Option<UserCountry>,
    vat_id: Option<UserVatId>,
}

async fn update(
    user_service: State<Arc<impl UserService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
    Json(UpdateRequest {
        name,
        display_name,
        email,
        email_verified,
        password,
        enabled,
        admin,
        description,
        tags,
        newsletter,
        business,
        first_name,
        last_name,
        street,
        zip_code,
        city,
        country,
        vat_id,
    }): Json<UpdateRequest>,
) -> Response {
    match user_service
        .update_user(
            &token.0,
            user_id.into(),
            UserUpdateRequest {
                user: UserUpdateUserRequest {
                    name: name.into(),
                    email: email.into(),
                    email_verified: email_verified.into(),
                    password: password
                        .map(|pw| match pw {
                            ApiUserPasswordOrEmpty::Empty => PasswordUpdate::Remove,
                            ApiUserPasswordOrEmpty::Password(pw) => PasswordUpdate::Change(pw),
                        })
                        .into(),
                    enabled: enabled.into(),
                    admin: admin.into(),
                    newsletter: newsletter.into(),
                },
                profile: UserProfilePatch {
                    display_name: display_name.into(),
                    bio: description.into(),
                    tags: tags.into(),
                },
                invoice_info: UserInvoiceInfo {
                    business,
                    first_name,
                    last_name,
                    street,
                    zip_code,
                    city,
                    country,
                    vat_id,
                },
            },
        )
        .await
    {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(UserUpdateError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(UserUpdateError::NameConflict) => error(StatusCode::CONFLICT, "User already exists"),
        Err(UserUpdateError::EmailConflict) => error(StatusCode::CONFLICT, "Email already exists"),
        Err(UserUpdateError::CannotRemovePassword) => {
            error(StatusCode::FORBIDDEN, "Cannot delete last login method")
        }
        Err(
            UserUpdateError::CannotDisableSelf
            | UserUpdateError::CannotDemoteSelf
            | UserUpdateError::NameChangeRateLimit { .. },
        ) => error(StatusCode::FORBIDDEN, "Permission denied"),
        Err(UserUpdateError::NoEmail) => error(StatusCode::FORBIDDEN, "No email"),
        Err(UserUpdateError::InvalidVatId) => error(StatusCode::NOT_FOUND, "Invalid VAT ID"),
        Err(UserUpdateError::Auth(err)) => auth_error(err),
        Err(UserUpdateError::Other(err)) => internal_server_error(err),
    }
}

async fn delete(
    user_service: State<Arc<impl UserService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
) -> Response {
    match user_service.delete_user(&token.0, user_id.into()).await {
        Ok(()) => Json(true).into_response(),
        Err(UserDeleteError::NotFound) => error(StatusCode::NOT_FOUND, "User not found"),
        Err(UserDeleteError::Auth(err)) => auth_error(err),
        Err(UserDeleteError::Other(err)) => internal_server_error(err),
    }
}

async fn request_verification_email(
    service: State<Arc<impl UserService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
) -> Response {
    match service
        .request_verification_email(&token.0, user_id.into())
        .await
    {
        Ok(()) => Json(true).into_response(),
        Err(UserRequestVerificationEmailError::NotFound) => {
            error(StatusCode::NOT_FOUND, "User not found")
        }
        Err(UserRequestVerificationEmailError::AlreadyVerified) => {
            error(StatusCode::PRECONDITION_FAILED, "Email already verified")
        }
        Err(UserRequestVerificationEmailError::NoEmail) => {
            error(StatusCode::BAD_REQUEST, "Invalid email")
        }
        Err(UserRequestVerificationEmailError::Auth(err)) => auth_error(err),
        Err(UserRequestVerificationEmailError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct VerifyEmailRequest {
    code: VerificationCode,
}

async fn verify_email(
    service: State<Arc<impl UserService>>,
    Path(user_id): Path<ApiUserIdOrSelf>,
    Json(VerifyEmailRequest { code }): Json<VerifyEmailRequest>,
) -> Response {
    let ApiUserIdOrSelf::Slf = user_id else {
        return error(StatusCode::BAD_REQUEST, "Can only verify email for self");
    };

    match service.verify_email(code).await {
        Ok(()) => Json(true).into_response(),
        Err(UserVerifyEmailError::InvalidCode) => {
            error(StatusCode::UNAUTHORIZED, "Invalid verification code")
        }
        Err(UserVerifyEmailError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct VerifyNewsletterSubscriptionRequest {
    code: VerificationCode,
}

async fn verify_newsletter_subscription(
    service: State<Arc<impl UserService>>,
    token: ApiToken,
    Path(user_id): Path<ApiUserIdOrSelf>,
    Json(VerifyNewsletterSubscriptionRequest { code }): Json<VerifyNewsletterSubscriptionRequest>,
) -> Response {
    match service
        .verify_newsletter_subscription(&token.0, user_id.into(), code)
        .await
    {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(UserVerifyNewsletterSubscriptionError::NotFound) => {
            error(StatusCode::NOT_FOUND, "User not found")
        }
        Err(UserVerifyNewsletterSubscriptionError::AlreadySubscribed) => {
            error(StatusCode::CONFLICT, "Newsletter already subscribed")
        }
        Err(UserVerifyNewsletterSubscriptionError::InvalidCode) => {
            error(StatusCode::UNAUTHORIZED, "Invalid verification code")
        }
        Err(UserVerifyNewsletterSubscriptionError::Auth(err)) => auth_error(err),
        Err(UserVerifyNewsletterSubscriptionError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct RequestPasswordResetRequest {
    email: EmailAddress,
    recaptcha_response: Option<RecaptchaResponse>,
}

async fn request_password_reset(
    service: State<Arc<impl UserService>>,
    Json(RequestPasswordResetRequest {
        email,
        recaptcha_response,
    }): Json<RequestPasswordResetRequest>,
) -> Response {
    match service
        .request_password_reset(email, recaptcha_response)
        .await
    {
        Ok(()) => Json(true).into_response(),
        Err(UserRequestPasswordResetError::Recaptcha) => {
            error(StatusCode::PRECONDITION_FAILED, "Recaptcha failed")
        }
        Err(UserRequestPasswordResetError::Other(err)) => internal_server_error(err),
    }
}

#[derive(Deserialize)]
struct ResetPasswordRequest {
    email: EmailAddress,
    code: VerificationCode,
    password: UserPassword,
}

async fn reset_password(
    service: State<Arc<impl UserService>>,
    Json(ResetPasswordRequest {
        email,
        code,
        password,
    }): Json<ResetPasswordRequest>,
) -> Response {
    match service.reset_password(email, code, password).await {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(UserResetPasswordError::Failed) => {
            error(StatusCode::UNAUTHORIZED, "Password reset failed")
        }
        Err(UserResetPasswordError::Other(err)) => internal_server_error(err),
    }
}
