use academy_core_auth_contracts::MockAuthService;
use academy_core_mfa_contracts::commands::{
    confirm_totp_device::MockMfaConfirmTotpDeviceCommandService,
    create_totp_device::MockMfaCreateTotpDeviceCommandService,
    disable::MockMfaDisableCommandService, reset_totp_device::MockMfaResetTotpDeviceCommandService,
    setup_recovery::MockMfaSetupRecoveryCommandService,
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
    MockMfaCreateTotpDeviceCommandService<MockTransaction>,
    MockMfaResetTotpDeviceCommandService<MockTransaction>,
    MockMfaConfirmTotpDeviceCommandService<MockTransaction>,
    MockMfaSetupRecoveryCommandService<MockTransaction>,
    MockMfaDisableCommandService<MockTransaction>,
>;
