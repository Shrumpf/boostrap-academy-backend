use academy_core_internal_contracts::auth::MockInternalAuthService;
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase, MockTransaction};

use crate::InternalServiceImpl;

mod get_user;
mod get_user_by_email;

type Sut =
    InternalServiceImpl<MockDatabase, MockInternalAuthService, MockUserRepository<MockTransaction>>;
