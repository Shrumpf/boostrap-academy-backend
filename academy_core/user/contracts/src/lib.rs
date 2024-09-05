use std::future::Future;

use academy_models::{
    auth::{AuthError, Login},
    email_address::EmailAddress,
    oauth2::OAuth2RegistrationToken,
    session::DeviceName,
    user::{
        UserComposite, UserDisplayName, UserIdOrSelf, UserName, UserPassword, UserProfilePatch,
    },
    RecaptchaResponse, VerificationCode,
};
use academy_utils::patch::PatchValue;
use chrono::{DateTime, Utc};
use queries::list::{UserListQuery, UserListResult};
use thiserror::Error;

pub mod commands;
pub mod queries;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserService: Send + Sync + 'static {
    /// Returns a list of all users matching the given query.
    ///
    /// Can only be used by administrators.
    fn list_users(
        &self,
        token: &str,
        query: UserListQuery,
    ) -> impl Future<Output = Result<UserListResult, UserListError>> + Send;

    /// Returns the user with the given id.
    ///
    /// Can only be used by administrators, if not used on the authenticated
    /// user.
    fn get_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> impl Future<Output = Result<UserComposite, UserGetError>> + Send;

    /// Creates a new user and logs them in.
    fn create_user(
        &self,
        cmd: UserCreateRequest,
        device_name: Option<DeviceName>,
        recaptcha_response: Option<RecaptchaResponse>,
    ) -> impl Future<Output = Result<Login, UserCreateError>> + Send;

    /// Updates a user.
    ///
    /// - Changing the email address will also set `email_verified` to `false`.
    /// - Disabling a user will also log them out.
    /// - A user can never change their own admin status.
    /// - A user can never disable themselves.
    ///
    /// If the authenticated user is not an administrator:
    /// - Only the authenticated user itself can be updated.
    /// - Changing the `name` is rate-limited.
    /// - Changing any of the following fields is not allowed:
    ///   - `enabled`
    ///   - `admin`
    ///   - `email_verified`
    fn update_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        request: UserUpdateRequest,
    ) -> impl Future<Output = Result<UserComposite, UserUpdateError>> + Send;

    /// Deletes a user.
    ///
    /// Can only be used by administrators, if not used on the authenticated
    /// user.
    fn delete_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> impl Future<Output = Result<(), UserDeleteError>> + Send;

    /// Requests an email with a verification code to verify a user's email
    /// address.
    ///
    /// Can only be used by administrators, if not used on the authenticated
    /// user.
    fn request_verification_email(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> impl Future<Output = Result<(), UserRequestVerificationEmailError>> + Send;

    /// Verifies a user's email address using the verification code.
    fn verify_email(
        &self,
        code: VerificationCode,
    ) -> impl Future<Output = Result<(), UserVerifyEmailError>> + Send;

    /// Verifies the newsletter subscription using the verification code sent
    /// via email.
    fn verify_newsletter_subscription(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        code: VerificationCode,
    ) -> impl Future<Output = Result<UserComposite, UserVerifyNewsletterSubscriptionError>> + Send;

    fn request_password_reset(
        &self,
        email: EmailAddress,
        recaptcha_response: Option<RecaptchaResponse>,
    ) -> impl Future<Output = Result<(), UserRequestPasswordResetError>> + Send;

    fn reset_password(
        &self,
        email: EmailAddress,
        code: VerificationCode,
        new_password: UserPassword,
    ) -> impl Future<Output = Result<UserComposite, UserResetPasswordError>> + Send;
}

#[derive(Debug, Error)]
pub enum UserListError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UserGetError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct UserCreateRequest {
    pub name: UserName,
    pub display_name: UserDisplayName,
    pub email: EmailAddress,
    pub password: Option<UserPassword>,
    pub oauth2_registration_token: Option<OAuth2RegistrationToken>,
}

#[derive(Debug, Error)]
pub enum UserCreateError {
    #[error("A user with the same name already exists.")]
    NameConflict,
    #[error("A user with the same email address already exists.")]
    EmailConflict,
    #[error("Invalid recaptcha response")]
    Recaptcha,
    #[error("No login method has been provided.")]
    NoLoginMethod,
    #[error("The oauth registration token is invalid or has expired.")]
    InvalidOAuthRegistrationToken,
    #[error("The remote user has already been linked.")]
    RemoteAlreadyLinked,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Default)]
pub struct UserUpdateRequest {
    pub user: UserUpdateUserRequest,
    pub profile: UserProfilePatch,
}

#[derive(Debug, Default)]
pub struct UserUpdateUserRequest {
    pub name: PatchValue<UserName>,
    pub email: PatchValue<EmailAddress>,
    pub email_verified: PatchValue<bool>,
    pub password: PatchValue<PasswordUpdate>,
    pub enabled: PatchValue<bool>,
    pub admin: PatchValue<bool>,
    pub newsletter: PatchValue<bool>,
}

#[derive(Debug)]
pub enum PasswordUpdate {
    Change(UserPassword),
    Remove,
}

#[derive(Debug, Error)]
pub enum UserUpdateError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("The user does not exist.")]
    NotFound,
    #[error("A user with the same name already exists.")]
    NameConflict,
    #[error("A user with the same email address already exists.")]
    EmailConflict,
    #[error(
        "The password cannot be removed from the user because they don't have any other login \
         methods."
    )]
    CannotRemovePassword,
    #[error("The user cannot disable their own account.")]
    CannotDisableSelf,
    #[error("The user cannot change their own admin status.")]
    CannotDemoteSelf,
    #[error("The user cannot change their name until {until}.")]
    NameChangeRateLimit { until: DateTime<Utc> },
    #[error("The user does not have an email address.")]
    NoEmail,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UserDeleteError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("The user does not exist.")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UserRequestVerificationEmailError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("The user does not exist.")]
    NotFound,
    #[error("The user's email address has already been verified.")]
    AlreadyVerified,
    #[error("The user does not have an email address.")]
    NoEmail,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UserVerifyEmailError {
    #[error("The verification code is invalid.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UserVerifyNewsletterSubscriptionError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("The user does not exist.")]
    NotFound,
    #[error("The user is already subscribed to the newsletter.")]
    AlreadySubscribed,
    #[error("The verification code is incorrect.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UserRequestPasswordResetError {
    #[error("Invalid recaptcha response")]
    Recaptcha,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UserResetPasswordError {
    #[error("The email or verification code is invalid.")]
    Failed,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
