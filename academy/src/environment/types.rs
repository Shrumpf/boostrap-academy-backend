use academy_auth_impl::{
    access_token::AuthAccessTokenServiceImpl, internal::AuthInternalServiceImpl,
    refresh_token::AuthRefreshTokenServiceImpl, AuthServiceImpl,
};
use academy_cache_valkey::ValkeyCache;
use academy_core_config_impl::ConfigFeatureServiceImpl;
use academy_core_contact_impl::ContactFeatureServiceImpl;
use academy_core_health_impl::HealthFeatureServiceImpl;
use academy_core_internal_impl::InternalServiceImpl;
use academy_core_mfa_impl::{
    authenticate::MfaAuthenticateServiceImpl, disable::MfaDisableServiceImpl,
    recovery::MfaRecoveryServiceImpl, totp_device::MfaTotpDeviceServiceImpl, MfaFeatureServiceImpl,
};
use academy_core_oauth2_impl::{
    create_link::OAuth2CreateLinkServiceImpl, login::OAuth2LoginServiceImpl,
    OAuth2FeatureServiceImpl,
};
use academy_core_session_impl::{
    failed_auth_count::SessionFailedAuthCountServiceImpl, session::SessionServiceImpl,
    SessionFeatureServiceImpl,
};
use academy_core_user_impl::{
    commands::{
        create::UserCreateCommandServiceImpl,
        request_password_reset_email::UserRequestPasswordResetEmailCommandServiceImpl,
        request_subscribe_newsletter_email::UserRequestSubscribeNewsletterEmailCommandServiceImpl,
        request_verification_email::UserRequestVerificationEmailCommandServiceImpl,
        reset_password::UserResetPasswordCommandServiceImpl,
        update_admin::UserUpdateAdminCommandServiceImpl,
        update_email::UserUpdateEmailCommandServiceImpl,
        update_enabled::UserUpdateEnabledCommandServiceImpl,
        update_name::UserUpdateNameCommandServiceImpl,
        update_password::UserUpdatePasswordCommandServiceImpl,
        verify_email::UserVerifyEmailCommandServiceImpl,
        verify_newsletter_subscription::UserVerifyNewsletterSubscriptionCommandServiceImpl,
    },
    queries::{
        get_by_name_or_email::UserGetByNameOrEmailQueryServiceImpl, list::UserListQueryServiceImpl,
    },
    update_invoice_info::UserUpdateInvoiceInfoServiceImpl,
    UserFeatureServiceImpl,
};
use academy_email_impl::{template::TemplateEmailServiceImpl, EmailServiceImpl};
use academy_extern_impl::{
    internal::InternalApiServiceImpl, oauth2::OAuth2ApiServiceImpl,
    recaptcha::RecaptchaApiServiceImpl, vat::VatApiServiceImpl,
};
use academy_persistence_postgres::{
    mfa::PostgresMfaRepository, oauth2::PostgresOAuth2Repository,
    session::PostgresSessionRepository, user::PostgresUserRepository, PostgresDatabase,
};
use academy_shared_impl::{
    captcha::CaptchaServiceImpl, hash::HashServiceImpl, id::IdServiceImpl, jwt::JwtServiceImpl,
    password::PasswordServiceImpl, secret::SecretServiceImpl, time::TimeServiceImpl,
    totp::TotpServiceImpl,
};
use academy_templates_impl::TemplateServiceImpl;

// API
pub type RestServer = academy_api_rest::RestServer<
    HealthFeature,
    ConfigFeature,
    UserFeature,
    SessionFeature,
    ContactFeature,
    MfaFeature,
    OAuth2Feature,
    Internal,
>;

// Persistence
pub type Database = PostgresDatabase;

// Cache
pub type Cache = ValkeyCache;

// Email
pub type Email = EmailServiceImpl;
pub type TemplateEmail = TemplateEmailServiceImpl<Email, Template>;

// Extern
pub type RecaptchaApi = RecaptchaApiServiceImpl;
pub type OAuth2Api = OAuth2ApiServiceImpl;
pub type InternalApi = InternalApiServiceImpl<AuthInternal>;
pub type VatApi = VatApiServiceImpl;

// Template
pub type Template = TemplateServiceImpl;

// Shared
pub type Captcha = CaptchaServiceImpl<RecaptchaApi>;
pub type Hash = HashServiceImpl;
pub type Id = IdServiceImpl;
pub type Jwt = JwtServiceImpl<Time>;
pub type Password = PasswordServiceImpl;
pub type Secret = SecretServiceImpl;
pub type Time = TimeServiceImpl;
pub type Totp = TotpServiceImpl<Secret, Time, Hash, Cache>;

// Repositories
pub type SessionRepo = PostgresSessionRepository;
pub type UserRepo = PostgresUserRepository;
pub type MfaRepo = PostgresMfaRepository;
pub type OAuth2Repo = PostgresOAuth2Repository;

// Auth
pub type Auth =
    AuthServiceImpl<Time, Password, UserRepo, SessionRepo, AuthAccessToken, AuthRefreshToken>;
pub type AuthAccessToken = AuthAccessTokenServiceImpl<Jwt, Cache>;
pub type AuthRefreshToken = AuthRefreshTokenServiceImpl<Secret, Hash>;
pub type AuthInternal = AuthInternalServiceImpl<Jwt>;

// Core
pub type HealthFeature = HealthFeatureServiceImpl<Time, Database, Cache, Email>;

pub type ConfigFeature = ConfigFeatureServiceImpl<Captcha>;

pub type UserFeature = UserFeatureServiceImpl<
    Database,
    Auth,
    Cache,
    Captcha,
    VatApi,
    InternalApi,
    UserList,
    UserCreate,
    UserRequestSubscribeNewsletterEmail,
    UserUpdateName,
    UserUpdateEmail,
    UserUpdateAdmin,
    UserUpdateEnabled,
    UserUpdatePassword,
    UserVerifyNewsletterSubscription,
    UserRequestVerificationEmail,
    UserVerifyEmail,
    UserRequestPasswordResetEmail,
    UserResetPassword,
    UserUpdateInvoiceInfo,
    Session,
    UserRepo,
>;
pub type UserCreate = UserCreateCommandServiceImpl<Id, Time, Password, UserRepo, OAuth2CreateLink>;
pub type UserRequestSubscribeNewsletterEmail =
    UserRequestSubscribeNewsletterEmailCommandServiceImpl<Secret, TemplateEmail, Cache>;
pub type UserUpdateName = UserUpdateNameCommandServiceImpl<Time, UserRepo>;
pub type UserUpdateEmail = UserUpdateEmailCommandServiceImpl<Auth, UserRepo>;
pub type UserUpdateAdmin = UserUpdateAdminCommandServiceImpl<Auth, UserRepo>;
pub type UserUpdateEnabled = UserUpdateEnabledCommandServiceImpl<UserRepo, Session>;
pub type UserUpdatePassword = UserUpdatePasswordCommandServiceImpl<Password, UserRepo>;
pub type UserGetByNameOrEmail = UserGetByNameOrEmailQueryServiceImpl<UserRepo>;
pub type UserVerifyNewsletterSubscription =
    UserVerifyNewsletterSubscriptionCommandServiceImpl<UserRepo, Cache>;
pub type UserRequestVerificationEmail =
    UserRequestVerificationEmailCommandServiceImpl<Secret, TemplateEmail, Cache>;
pub type UserRequestPasswordResetEmail =
    UserRequestPasswordResetEmailCommandServiceImpl<Secret, TemplateEmail, Cache>;
pub type UserResetPassword = UserResetPasswordCommandServiceImpl<Cache, Password, UserRepo>;
pub type UserVerifyEmail = UserVerifyEmailCommandServiceImpl<Auth, Cache, UserRepo>;
pub type UserUpdateInvoiceInfo = UserUpdateInvoiceInfoServiceImpl<UserRepo>;
pub type UserList = UserListQueryServiceImpl<UserRepo>;

pub type SessionFeature = SessionFeatureServiceImpl<
    Database,
    Auth,
    Captcha,
    Session,
    SessionFailedAuthCount,
    UserGetByNameOrEmail,
    MfaAuthenticate,
    UserRepo,
    SessionRepo,
>;
pub type Session = SessionServiceImpl<Id, Time, Auth, AuthAccessToken, SessionRepo, UserRepo>;
pub type SessionFailedAuthCount = SessionFailedAuthCountServiceImpl<Hash, Cache>;

pub type ContactFeature = ContactFeatureServiceImpl<Captcha, Email>;

pub type MfaFeature = MfaFeatureServiceImpl<
    Database,
    Auth,
    UserRepo,
    MfaRepo,
    MfaRecovery,
    MfaDisable,
    MfaTotpDevice,
>;
pub type MfaRecovery = MfaRecoveryServiceImpl<Secret, Hash, MfaRepo>;
pub type MfaAuthenticate = MfaAuthenticateServiceImpl<Hash, Totp, MfaDisable, MfaRepo>;
pub type MfaDisable = MfaDisableServiceImpl<MfaRepo>;
pub type MfaTotpDevice = MfaTotpDeviceServiceImpl<Id, Time, Totp, MfaRepo>;

pub type OAuth2Feature = OAuth2FeatureServiceImpl<
    Database,
    Auth,
    Cache,
    Secret,
    OAuth2Api,
    UserRepo,
    OAuth2Repo,
    OAuth2CreateLink,
    OAuth2Login,
    Session,
>;
pub type OAuth2CreateLink = OAuth2CreateLinkServiceImpl<Id, Time, OAuth2Repo>;
pub type OAuth2Login = OAuth2LoginServiceImpl<OAuth2Api>;

pub type Internal = InternalServiceImpl<Database, AuthInternal, UserRepo>;
