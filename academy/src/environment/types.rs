use academy_cache_valkey::ValkeyCache;
use academy_core_auth_impl::{
    commands::invalidate_access_token::AuthInvalidateAccessTokenCommandServiceImpl, AuthServiceImpl,
};
use academy_core_config_impl::ConfigServiceImpl;
use academy_core_contact_impl::ContactServiceImpl;
use academy_core_health_impl::HealthServiceImpl;
use academy_core_internal_impl::{auth::InternalAuthServiceImpl, InternalServiceImpl};
use academy_core_mfa_impl::{
    commands::{
        authenticate::MfaAuthenticateCommandServiceImpl,
        confirm_totp_device::MfaConfirmTotpDeviceCommandServiceImpl,
        create_totp_device::MfaCreateTotpDeviceCommandServiceImpl,
        disable::MfaDisableCommandServiceImpl,
        reset_totp_device::MfaResetTotpDeviceCommandServiceImpl,
        setup_recovery::MfaSetupRecoveryCommandServiceImpl,
    },
    MfaServiceImpl,
};
use academy_core_oauth2_impl::{
    create_link::OAuth2CreateLinkServiceImpl, login::OAuth2LoginServiceImpl, OAuth2ServiceImpl,
};
use academy_core_session_impl::{
    commands::{
        create::SessionCreateCommandServiceImpl, delete::SessionDeleteCommandServiceImpl,
        delete_by_user::SessionDeleteByUserCommandServiceImpl,
        refresh::SessionRefreshCommandServiceImpl,
    },
    failed_auth_count::SessionFailedAuthCountServiceImpl,
    SessionServiceImpl,
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
    UserServiceImpl,
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
pub type RestServer =
    academy_api_rest::RestServer<Health, Config, User, Session, Contact, Mfa, OAuth2, Internal>;

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
pub type InternalApi = InternalApiServiceImpl<Jwt>;
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

// Core
pub type Auth = AuthServiceImpl<
    Jwt,
    Secret,
    Time,
    Hash,
    Password,
    UserRepo,
    SessionRepo,
    Cache,
    AuthInvalidateAccessToken,
>;
pub type AuthInvalidateAccessToken = AuthInvalidateAccessTokenCommandServiceImpl<Cache>;

pub type Health = HealthServiceImpl<Time, Database, Cache, Email>;

pub type Config = ConfigServiceImpl<Captcha>;

pub type User = UserServiceImpl<
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
    SessionCreate,
    UserRepo,
>;
pub type UserCreate = UserCreateCommandServiceImpl<Id, Time, Password, UserRepo, OAuth2CreateLink>;
pub type UserRequestSubscribeNewsletterEmail =
    UserRequestSubscribeNewsletterEmailCommandServiceImpl<Secret, TemplateEmail, Cache>;
pub type UserUpdateName = UserUpdateNameCommandServiceImpl<Time, UserRepo>;
pub type UserUpdateEmail = UserUpdateEmailCommandServiceImpl<Auth, UserRepo>;
pub type UserUpdateAdmin = UserUpdateAdminCommandServiceImpl<Auth, UserRepo>;
pub type UserUpdateEnabled = UserUpdateEnabledCommandServiceImpl<UserRepo, SessionDeleteByUser>;
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

pub type Session = SessionServiceImpl<
    Database,
    Auth,
    Captcha,
    SessionCreate,
    SessionRefresh,
    SessionDelete,
    SessionDeleteByUser,
    SessionFailedAuthCount,
    UserGetByNameOrEmail,
    MfaAuthenticate,
    UserRepo,
    SessionRepo,
>;
pub type SessionCreate = SessionCreateCommandServiceImpl<Id, Time, Auth, SessionRepo, UserRepo>;
pub type SessionRefresh = SessionRefreshCommandServiceImpl<Time, Auth, UserRepo, SessionRepo>;
pub type SessionDelete = SessionDeleteCommandServiceImpl<Auth, SessionRepo>;
pub type SessionDeleteByUser = SessionDeleteByUserCommandServiceImpl<Auth, SessionRepo>;
pub type SessionFailedAuthCount = SessionFailedAuthCountServiceImpl<Hash, Cache>;

pub type Contact = ContactServiceImpl<Captcha, Email>;

pub type Mfa = MfaServiceImpl<
    Database,
    Auth,
    UserRepo,
    MfaRepo,
    MfaCreateTotpDevice,
    MfaResetTotpDevice,
    MfaConfirmTotpDevice,
    MfaSetupRecovery,
    MfaDisable,
>;
pub type MfaCreateTotpDevice = MfaCreateTotpDeviceCommandServiceImpl<Id, Time, Totp, MfaRepo>;
pub type MfaResetTotpDevice = MfaResetTotpDeviceCommandServiceImpl<Totp, MfaRepo>;
pub type MfaConfirmTotpDevice = MfaConfirmTotpDeviceCommandServiceImpl<Totp, MfaRepo>;
pub type MfaSetupRecovery = MfaSetupRecoveryCommandServiceImpl<Secret, Hash, MfaRepo>;
pub type MfaAuthenticate = MfaAuthenticateCommandServiceImpl<Hash, Totp, MfaDisable, MfaRepo>;
pub type MfaDisable = MfaDisableCommandServiceImpl<MfaRepo>;

pub type OAuth2 = OAuth2ServiceImpl<
    Database,
    Auth,
    Cache,
    Secret,
    OAuth2Api,
    UserRepo,
    OAuth2Repo,
    OAuth2CreateLink,
    OAuth2Login,
    SessionCreate,
>;
pub type OAuth2CreateLink = OAuth2CreateLinkServiceImpl<Id, Time, OAuth2Repo>;
pub type OAuth2Login = OAuth2LoginServiceImpl<OAuth2Api>;

pub type Internal = InternalServiceImpl<Database, InternalAuth, UserRepo>;
pub type InternalAuth = InternalAuthServiceImpl<Jwt>;
