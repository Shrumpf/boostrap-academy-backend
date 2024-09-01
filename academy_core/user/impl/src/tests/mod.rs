use academy_core_auth_contracts::MockAuthService;
use academy_core_session_contracts::commands::create::MockSessionCreateCommandService;
use academy_core_user_contracts::{
    commands::{
        create::MockUserCreateCommandService,
        request_password_reset_email::MockUserRequestPasswordResetEmailCommandService,
        request_subscribe_newsletter_email::MockUserRequestSubscribeNewsletterEmailCommandService,
        request_verification_email::MockUserRequestVerificationEmailCommandService,
        reset_password::MockUserResetPasswordCommandService,
        update_admin::MockUserUpdateAdminCommandService,
        update_email::MockUserUpdateEmailCommandService,
        update_enabled::MockUserUpdateEnabledCommandService,
        update_name::MockUserUpdateNameCommandService,
        update_password::MockUserUpdatePasswordCommandService,
        verify_email::MockUserVerifyEmailCommandService,
        verify_newsletter_subscription::MockUserVerifyNewsletterSubscriptionCommandService,
    },
    queries::list::MockUserListQueryService,
};
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase, MockTransaction};
use academy_shared_contracts::captcha::MockCaptchaService;

use crate::UserServiceImpl;

mod create_user;
mod delete_user;
mod get_user;
mod list_users;
mod request_password_reset;
mod request_verification_email;
mod reset_password;
mod update_user;
mod verify_email;
mod verify_newsletter_subscription;

type Sut = UserServiceImpl<
    MockDatabase,
    MockAuthService<MockTransaction>,
    MockCaptchaService,
    MockUserListQueryService<MockTransaction>,
    MockUserCreateCommandService<MockTransaction>,
    MockUserRequestSubscribeNewsletterEmailCommandService,
    MockUserUpdateNameCommandService<MockTransaction>,
    MockUserUpdateEmailCommandService<MockTransaction>,
    MockUserUpdateAdminCommandService<MockTransaction>,
    MockUserUpdateEnabledCommandService<MockTransaction>,
    MockUserUpdatePasswordCommandService<MockTransaction>,
    MockUserVerifyNewsletterSubscriptionCommandService<MockTransaction>,
    MockUserRequestVerificationEmailCommandService,
    MockUserVerifyEmailCommandService<MockTransaction>,
    MockUserRequestPasswordResetEmailCommandService,
    MockUserResetPasswordCommandService<MockTransaction>,
    MockSessionCreateCommandService<MockTransaction>,
    MockUserRepository<MockTransaction>,
>;
