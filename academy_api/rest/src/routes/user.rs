use std::sync::Arc;

use academy_core_user_contracts::{
    user::{UserListQuery, UserListResult},
    PasswordUpdate, UserCreateError, UserCreateRequest, UserDeleteError, UserFeatureService,
    UserGetError, UserListError, UserRequestPasswordResetError, UserRequestVerificationEmailError,
    UserResetPasswordError, UserUpdateError, UserUpdateRequest, UserUpdateUserRequest,
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
use aide::{
    axum::{routing, ApiRouter},
    transform::TransformOperation,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    docs::TransformOperationExt,
    errors::{
        auth_error, auth_error_docs, error, internal_server_error, internal_server_error_docs,
        recaptcha_error_docs, ApiError, EmailAlreadyExistsDetail, InvalidOAuthTokenDetail,
        NoLoginMethodDetail, RecaptchaFailedDetail, RemoteAlreadyLinkedDetail,
        UserAlreadyExistsDetail, UserNotFoundDetail,
    },
    extractors::{auth::ApiToken, user_agent::UserAgent},
    models::{
        session::ApiLogin,
        user::{ApiUser, ApiUserFilter, ApiUserIdOrSelf, ApiUserPasswordOrEmpty, PathUserIdOrSelf},
        ApiPaginationSlice, StringOption,
    },
};

pub const TAG: &str = "User";

pub fn router(service: Arc<impl UserFeatureService>) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route(
            "/auth/users",
            routing::get_with(list, list_docs).post_with(create, create_docs),
        )
        .api_route(
            "/auth/users/:user_id",
            routing::get_with(get, get_docs)
                .patch_with(update, update_docs)
                .delete_with(delete, delete_docs),
        )
        .api_route(
            "/auth/users/:user_id/email",
            routing::post_with(request_verification_email, request_verification_email_docs)
                .put_with(verify_email, verify_email_docs),
        )
        .api_route(
            "/auth/users/:user_id/newsletter",
            routing::put_with(
                verify_newsletter_subscription,
                verify_newsletter_subscription_docs,
            ),
        )
        .api_route(
            "/auth/password_reset",
            routing::post_with(request_password_reset, request_password_reset_docs)
                .put_with(reset_password, reset_password_docs),
        )
        .with_state(service)
        .with_path_items(|op| op.tag(TAG))
}

#[derive(Serialize, JsonSchema)]
struct ListResult {
    /// The total number of users matching the given query
    total: u64,
    /// The paginated list of users matching the given query
    users: Vec<ApiUser>,
}

async fn list(
    user_service: State<Arc<impl UserFeatureService>>,
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

fn list_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Return a list of all users matching the given query.")
        .add_response::<ListResult>(StatusCode::OK, None)
        .with(auth_error_docs)
        .with(internal_server_error_docs)
}

async fn get(
    user_service: State<Arc<impl UserFeatureService>>,
    token: ApiToken,
    Path(PathUserIdOrSelf { user_id }): Path<PathUserIdOrSelf>,
) -> Response {
    match user_service.get_user(&token.0, user_id.into()).await {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(UserGetError::Auth(err)) => auth_error(err),
        Err(UserGetError::NotFound) => error(StatusCode::NOT_FOUND, UserNotFoundDetail),
        Err(UserGetError::Other(err)) => internal_server_error(err),
    }
}

fn get_docs(op: TransformOperation) -> TransformOperation {
    op
}

#[derive(Deserialize, JsonSchema)]
struct CreateRequest {
    name: UserName,
    display_name: UserDisplayName,
    email: EmailAddress,
    password: StringOption<UserPassword>,
    oauth_register_token: StringOption<OAuth2RegistrationToken>,
    recaptcha_response: StringOption<RecaptchaResponse>,
}

async fn create(
    user_service: State<Arc<impl UserFeatureService>>,
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
                password: password.into(),
                oauth2_registration_token: oauth_register_token.into(),
            },
            user_agent.0.map(DeviceName::from_string_truncated),
            recaptcha_response.into(),
        )
        .await
    {
        Ok(result) => Json(ApiLogin::from(result)).into_response(),
        Err(UserCreateError::NameConflict) => error(StatusCode::CONFLICT, UserAlreadyExistsDetail),
        Err(UserCreateError::EmailConflict) => {
            error(StatusCode::CONFLICT, EmailAlreadyExistsDetail)
        }
        Err(UserCreateError::Recaptcha) => {
            error(StatusCode::PRECONDITION_FAILED, RecaptchaFailedDetail)
        }
        Err(UserCreateError::NoLoginMethod) => {
            error(StatusCode::PRECONDITION_FAILED, NoLoginMethodDetail)
        }
        Err(UserCreateError::InvalidOAuthRegistrationToken) => {
            error(StatusCode::UNAUTHORIZED, InvalidOAuthTokenDetail)
        }
        Err(UserCreateError::RemoteAlreadyLinked) => {
            error(StatusCode::CONFLICT, RemoteAlreadyLinkedDetail)
        }
        Err(UserCreateError::Other(err)) => internal_server_error(err),
    }
}

fn create_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Create a new user account.")
        .description("Also creates a session for the new user.")
        .add_response::<ApiLogin>(StatusCode::OK, None)
        .add_response::<ApiError<UserAlreadyExistsDetail>>(
            StatusCode::CONFLICT,
            "A user with this name already exists.",
        )
        .add_response::<ApiError<EmailAlreadyExistsDetail>>(
            StatusCode::CONFLICT,
            "A user with this email address already exists.",
        )
        .with(recaptcha_error_docs)
        .add_response::<ApiError<NoLoginMethodDetail>>(
            StatusCode::PRECONDITION_FAILED,
            "No login method was provided.",
        )
        .add_response::<ApiError<InvalidOAuthTokenDetail>>(
            StatusCode::UNAUTHORIZED,
            "The OAuth2 registration token is invalid or has expired.",
        )
        .add_response::<ApiError<RemoteAlreadyLinkedDetail>>(
            StatusCode::CONFLICT,
            "The remote user has already been linked to another account.",
        )
        .with(internal_server_error_docs)
}

#[derive(Deserialize, JsonSchema)]
struct UpdateRequest {
    name: StringOption<UserName>,
    display_name: StringOption<UserDisplayName>,
    email: StringOption<EmailAddress>,
    email_verified: Option<bool>,
    password: Option<ApiUserPasswordOrEmpty>,
    enabled: Option<bool>,
    admin: Option<bool>,
    description: StringOption<UserBio>,
    tags: Option<UserTags>,
    newsletter: Option<bool>,
    business: Option<bool>,
    first_name: StringOption<UserFirstName>,
    last_name: StringOption<UserLastName>,
    street: StringOption<UserStreet>,
    zip_code: StringOption<UserZipCode>,
    city: StringOption<UserCity>,
    country: StringOption<UserCountry>,
    vat_id: StringOption<UserVatId>,
}

async fn update(
    user_service: State<Arc<impl UserFeatureService>>,
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
                    name: Option::from(name).into(),
                    email: Option::from(email).into(),
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
                    display_name: Option::from(display_name).into(),
                    bio: Option::from(description).into(),
                    tags: tags.into(),
                },
                invoice_info: UserInvoiceInfo {
                    business,
                    first_name: first_name.into(),
                    last_name: last_name.into(),
                    street: street.into(),
                    zip_code: zip_code.into(),
                    city: city.into(),
                    country: country.into(),
                    vat_id: vat_id.into(),
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

fn update_docs(op: TransformOperation) -> TransformOperation {
    op
}

async fn delete(
    user_service: State<Arc<impl UserFeatureService>>,
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

fn delete_docs(op: TransformOperation) -> TransformOperation {
    op
}

async fn request_verification_email(
    service: State<Arc<impl UserFeatureService>>,
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

fn request_verification_email_docs(op: TransformOperation) -> TransformOperation {
    op
}

#[derive(Deserialize, JsonSchema)]
struct VerifyEmailRequest {
    code: VerificationCode,
}

async fn verify_email(
    service: State<Arc<impl UserFeatureService>>,
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

fn verify_email_docs(op: TransformOperation) -> TransformOperation {
    op
}

#[derive(Deserialize, JsonSchema)]
struct VerifyNewsletterSubscriptionRequest {
    code: VerificationCode,
}

async fn verify_newsletter_subscription(
    service: State<Arc<impl UserFeatureService>>,
    token: ApiToken,
    Path(PathUserIdOrSelf { user_id }): Path<PathUserIdOrSelf>,
    Json(VerifyNewsletterSubscriptionRequest { code }): Json<VerifyNewsletterSubscriptionRequest>,
) -> Response {
    match service
        .verify_newsletter_subscription(&token.0, user_id.into(), code)
        .await
    {
        Ok(user) => Json(ApiUser::from(user)).into_response(),
        Err(UserVerifyNewsletterSubscriptionError::NotFound) => {
            error(StatusCode::NOT_FOUND, UserNotFoundDetail)
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

fn verify_newsletter_subscription_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Verify the newsletter subscription using a verification code.")
        .add_response::<ApiUser>(StatusCode::OK, None)
}

#[derive(Deserialize, JsonSchema)]
struct RequestPasswordResetRequest {
    email: EmailAddress,
    recaptcha_response: StringOption<RecaptchaResponse>,
}

async fn request_password_reset(
    service: State<Arc<impl UserFeatureService>>,
    Json(RequestPasswordResetRequest {
        email,
        recaptcha_response,
    }): Json<RequestPasswordResetRequest>,
) -> Response {
    match service
        .request_password_reset(email, recaptcha_response.into())
        .await
    {
        Ok(()) => Json(true).into_response(),
        Err(UserRequestPasswordResetError::Recaptcha) => {
            error(StatusCode::PRECONDITION_FAILED, "Recaptcha failed")
        }
        Err(UserRequestPasswordResetError::Other(err)) => internal_server_error(err),
    }
}

fn request_password_reset_docs(op: TransformOperation) -> TransformOperation {
    op
}

#[derive(Deserialize, JsonSchema)]
struct ResetPasswordRequest {
    email: EmailAddress,
    code: VerificationCode,
    password: UserPassword,
}

async fn reset_password(
    service: State<Arc<impl UserFeatureService>>,
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

fn reset_password_docs(op: TransformOperation) -> TransformOperation {
    op
}
