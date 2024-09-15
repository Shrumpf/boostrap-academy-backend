use academy_auth_contracts::MockAuthService;
use academy_core_mfa_contracts::{
    disable::MockMfaDisableService, recovery::MockMfaRecoveryService,
    totp_device::MockMfaTotpDeviceService,
};
use academy_persistence_contracts::{
    mfa::MockMfaRepository, user::MockUserRepository, MockDatabase, MockTransaction,
};

use crate::MfaServiceImpl;

mod disable;
mod enable;
mod initialize;

type Sut = MfaServiceImpl<
    MockDatabase,
    MockAuthService<MockTransaction>,
    MockUserRepository<MockTransaction>,
    MockMfaRepository<MockTransaction>,
    MockMfaRecoveryService<MockTransaction>,
    MockMfaDisableService<MockTransaction>,
    MockMfaTotpDeviceService<MockTransaction>,
>;
