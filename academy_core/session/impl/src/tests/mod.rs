use academy_core_auth_contracts::MockAuthService;
use academy_core_mfa_contracts::commands::authenticate::MockMfaAuthenticateCommandService;
use academy_core_session_contracts::commands::{
    create::MockSessionCreateCommandService, delete::MockSessionDeleteCommandService,
    delete_by_user::MockSessionDeleteByUserCommandService,
    refresh::MockSessionRefreshCommandService,
};
use academy_core_user_contracts::queries::get_by_name_or_email::MockUserGetByNameOrEmailQueryService;
use academy_persistence_contracts::{
    session::MockSessionRepository, user::MockUserRepository, MockDatabase, MockTransaction,
};

use crate::SessionServiceImpl;

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
    MockSessionCreateCommandService<MockTransaction>,
    MockSessionRefreshCommandService<MockTransaction>,
    MockSessionDeleteCommandService<MockTransaction>,
    MockSessionDeleteByUserCommandService<MockTransaction>,
    MockUserGetByNameOrEmailQueryService<MockTransaction>,
    MockMfaAuthenticateCommandService<MockTransaction>,
    MockUserRepository<MockTransaction>,
    MockSessionRepository<MockTransaction>,
>;
