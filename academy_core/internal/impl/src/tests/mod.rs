use academy_core_auth_contracts::internal::MockAuthInternalService;
use academy_persistence_contracts::{user::MockUserRepository, MockDatabase, MockTransaction};

use crate::InternalServiceImpl;

mod get_user;
mod get_user_by_email;

type Sut =
    InternalServiceImpl<MockDatabase, MockAuthInternalService, MockUserRepository<MockTransaction>>;
