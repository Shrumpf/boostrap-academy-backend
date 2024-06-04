use academy_core_auth_contracts::{AuthResultExt, AuthService};
use academy_core_mfa_contracts::{
    commands::{
        confirm_totp_device::{
            MfaConfirmTotpDeviceCommandError, MfaConfirmTotpDeviceCommandService,
        },
        create_totp_device::MfaCreateTotpDeviceCommandService,
        disable::MfaDisableCommandService,
        reset_totp_device::MfaResetTotpDeviceCommandService,
        setup_recovery::MfaSetupRecoveryCommandService,
    },
    MfaDisableError, MfaEnableError, MfaInitializeError, MfaService,
};
use academy_di::Build;
use academy_models::{
    mfa::{MfaRecoveryCode, TotpCode, TotpSetup},
    user::UserIdOrSelf,
};
use academy_persistence_contracts::{
    mfa::MfaRepository, user::UserRepository, Database, Transaction,
};

pub mod commands;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Build, Default)]
pub struct MfaServiceImpl<
    Db,
    Auth,
    UserRepo,
    MfaRepo,
    MfaCreateTotpDevice,
    MfaResetTotpDevice,
    MfaConfirmTotpDevice,
    MfaSetupRecovery,
    MfaDisable,
> {
    db: Db,
    auth: Auth,
    user_repo: UserRepo,
    mfa_repo: MfaRepo,
    mfa_create_totp_device: MfaCreateTotpDevice,
    mfa_reset_totp_device: MfaResetTotpDevice,
    mfa_confirm_totp_device: MfaConfirmTotpDevice,
    mfa_setup_recovery: MfaSetupRecovery,
    mfa_disable: MfaDisable,
}

impl<
        Db,
        Auth,
        UserRepo,
        MfaRepo,
        MfaCreateTotpDevice,
        MfaResetTotpDevice,
        MfaConfirmTotpDevice,
        MfaSetupRecovery,
        MfaDisable,
    > MfaService
    for MfaServiceImpl<
        Db,
        Auth,
        UserRepo,
        MfaRepo,
        MfaCreateTotpDevice,
        MfaResetTotpDevice,
        MfaConfirmTotpDevice,
        MfaSetupRecovery,
        MfaDisable,
    >
where
    Db: Database,
    Auth: AuthService<Db::Transaction>,
    UserRepo: UserRepository<Db::Transaction>,
    MfaRepo: MfaRepository<Db::Transaction>,
    MfaCreateTotpDevice: MfaCreateTotpDeviceCommandService<Db::Transaction>,
    MfaResetTotpDevice: MfaResetTotpDeviceCommandService<Db::Transaction>,
    MfaConfirmTotpDevice: MfaConfirmTotpDeviceCommandService<Db::Transaction>,
    MfaSetupRecovery: MfaSetupRecoveryCommandService<Db::Transaction>,
    MfaDisable: MfaDisableCommandService<Db::Transaction>,
{
    async fn initialize(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> Result<TotpSetup, MfaInitializeError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        if !self.user_repo.exists(&mut txn, user_id).await? {
            return Err(MfaInitializeError::NotFound);
        }

        let totp_devices = self
            .mfa_repo
            .list_totp_devices_by_user(&mut txn, user_id)
            .await?;

        if totp_devices.iter().any(|x| x.enabled) {
            return Err(MfaInitializeError::AlreadyEnabled);
        }

        let setup = if let Some(disabled_totp_device) = totp_devices.first() {
            self.mfa_reset_totp_device
                .invoke(&mut txn, disabled_totp_device.id)
                .await?
        } else {
            self.mfa_create_totp_device
                .invoke(&mut txn, user_id)
                .await?
        };

        txn.commit().await?;

        Ok(setup)
    }

    async fn enable(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        code: TotpCode,
    ) -> Result<MfaRecoveryCode, MfaEnableError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        if !self.user_repo.exists(&mut txn, user_id).await? {
            return Err(MfaEnableError::NotFound);
        }

        let totp_devices = self
            .mfa_repo
            .list_totp_devices_by_user(&mut txn, user_id)
            .await?;

        if totp_devices.iter().any(|x| x.enabled) {
            return Err(MfaEnableError::AlreadyEnabled);
        }

        let totp_device = totp_devices
            .into_iter()
            .next()
            .ok_or(MfaEnableError::NotInitialized)?;

        self.mfa_confirm_totp_device
            .invoke(&mut txn, totp_device, code)
            .await
            .map_err(|err| match err {
                MfaConfirmTotpDeviceCommandError::InvalidCode => MfaEnableError::InvalidCode,
                MfaConfirmTotpDeviceCommandError::Other(err) => err.into(),
            })?;

        let recovery_code = self.mfa_setup_recovery.invoke(&mut txn, user_id).await?;

        txn.commit().await?;

        Ok(recovery_code)
    }

    async fn disable(&self, token: &str, user_id: UserIdOrSelf) -> Result<(), MfaDisableError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        if !self.user_repo.exists(&mut txn, user_id).await? {
            return Err(MfaDisableError::NotFound);
        }

        let totp_devices = self
            .mfa_repo
            .list_totp_devices_by_user(&mut txn, user_id)
            .await?;

        if totp_devices.iter().all(|x| !x.enabled) {
            return Err(MfaDisableError::NotEnabled);
        }

        self.mfa_disable.invoke(&mut txn, user_id).await?;

        txn.commit().await?;

        Ok(())
    }
}
