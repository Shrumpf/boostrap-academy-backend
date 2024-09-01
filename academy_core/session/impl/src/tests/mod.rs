use academy_core_auth_contracts::MockAuthService;
use academy_core_mfa_contracts::commands::authenticate::MockMfaAuthenticateCommandService;
use academy_core_session_contracts::{
    commands::{
        create::MockSessionCreateCommandService, delete::MockSessionDeleteCommandService,
        delete_by_user::MockSessionDeleteByUserCommandService,
        refresh::MockSessionRefreshCommandService,
    },
    failed_auth_count::MockSessionFailedAuthCountService,
};
use academy_core_user_contracts::queries::get_by_name_or_email::MockUserGetByNameOrEmailQueryService;
use academy_persistence_contracts::{
    session::MockSessionRepository, user::MockUserRepository, MockDatabase, MockTransaction,
};
use academy_shared_contracts::captcha::MockCaptchaService;

use crate::{SessionServiceConfig, SessionServiceImpl};

mod create_session;
mod delete_by_user;
mod delete_current_session;
mod delete_session;
mod get_current_session;
mod impersonate;
mod list_by_user;
mod refresh;

type Sut = SessionServiceImpl<
    MockDatabase,
    MockAuthService<MockTransaction>,
    MockCaptchaService,
    MockSessionCreateCommandService<MockTransaction>,
    MockSessionRefreshCommandService<MockTransaction>,
    MockSessionDeleteCommandService<MockTransaction>,
    MockSessionDeleteByUserCommandService<MockTransaction>,
    MockSessionFailedAuthCountService,
    MockUserGetByNameOrEmailQueryService<MockTransaction>,
    MockMfaAuthenticateCommandService<MockTransaction>,
    MockUserRepository<MockTransaction>,
    MockSessionRepository<MockTransaction>,
>;

impl Default for SessionServiceConfig {
    fn default() -> Self {
        Self {
            login_fails_before_captcha: 3,
        }
    }
}
