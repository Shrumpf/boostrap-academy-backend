use academy_auth_contracts::{AuthResultExt, AuthService};
use academy_core_mfa_contracts::{
    disable::MfaDisableService,
    recovery::MfaRecoveryService,
    totp_device::{MfaTotpDeviceConfirmError, MfaTotpDeviceService},
    MfaDisableError, MfaEnableError, MfaFeatureService, MfaInitializeError,
};
use academy_di::Build;
use academy_models::{
    mfa::{MfaRecoveryCode, TotpCode, TotpSetup},
    user::UserIdOrSelf,
};
use academy_persistence_contracts::{
    mfa::MfaRepository, user::UserRepository, Database, Transaction,
};
use anyhow::Context;

pub mod authenticate;
pub mod disable;
pub mod recovery;
pub mod totp_device;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Build, Default)]
pub struct MfaFeatureServiceImpl<
    Db,
    Auth,
    UserRepo,
    MfaRepo,
    MfaRecovery,
    MfaDisable,
    MfaTotpDevice,
> {
    db: Db,
    auth: Auth,
    user_repo: UserRepo,
    mfa_repo: MfaRepo,
    mfa_recovery: MfaRecovery,
    mfa_disable: MfaDisable,
    mfa_totp_device: MfaTotpDevice,
}

impl<Db, Auth, UserRepo, MfaRepo, MfaRecovery, MfaDisable, MfaTotpDevice> MfaFeatureService
    for MfaFeatureServiceImpl<Db, Auth, UserRepo, MfaRepo, MfaRecovery, MfaDisable, MfaTotpDevice>
where
    Db: Database,
    Auth: AuthService<Db::Transaction>,
    UserRepo: UserRepository<Db::Transaction>,
    MfaRepo: MfaRepository<Db::Transaction>,
    MfaRecovery: MfaRecoveryService<Db::Transaction>,
    MfaDisable: MfaDisableService<Db::Transaction>,
    MfaTotpDevice: MfaTotpDeviceService<Db::Transaction>,
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

        if !self
            .user_repo
            .exists(&mut txn, user_id)
            .await
            .context("Failed to check user existence")?
        {
            return Err(MfaInitializeError::NotFound);
        }

        let totp_devices = self
            .mfa_repo
            .list_totp_devices_by_user(&mut txn, user_id)
            .await
            .context("Failed to get totp devices from database")?;

        if totp_devices.iter().any(|x| x.enabled) {
            return Err(MfaInitializeError::AlreadyEnabled);
        }

        let setup = if let Some(disabled_totp_device) = totp_devices.first() {
            self.mfa_totp_device
                .reset(&mut txn, disabled_totp_device.id)
                .await
                .with_context(|| {
                    format!("Failed to reset totp device {}", *disabled_totp_device.id)
                })?
        } else {
            self.mfa_totp_device
                .create(&mut txn, user_id)
                .await
                .context("Failed to create new totp device")?
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

        if !self
            .user_repo
            .exists(&mut txn, user_id)
            .await
            .context("Failed to check user existence")?
        {
            return Err(MfaEnableError::NotFound);
        }

        let totp_devices = self
            .mfa_repo
            .list_totp_devices_by_user(&mut txn, user_id)
            .await
            .context("Failed to get totp devices from database")?;

        if totp_devices.iter().any(|x| x.enabled) {
            return Err(MfaEnableError::AlreadyEnabled);
        }

        let totp_device = totp_devices
            .into_iter()
            .next()
            .ok_or(MfaEnableError::NotInitialized)?;

        let totp_device_id = totp_device.id;
        self.mfa_totp_device
            .confirm(&mut txn, totp_device, code)
            .await
            .map_err(|err| match err {
                MfaTotpDeviceConfirmError::InvalidCode => MfaEnableError::InvalidCode,
                MfaTotpDeviceConfirmError::Other(err) => err
                    .context(format!("Failed to confirm totp device {}", *totp_device_id))
                    .into(),
            })?;

        let recovery_code = self
            .mfa_recovery
            .setup(&mut txn, user_id)
            .await
            .context("Failed to setup recovery code")?;

        txn.commit().await?;

        Ok(recovery_code)
    }

    async fn disable(&self, token: &str, user_id: UserIdOrSelf) -> Result<(), MfaDisableError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        if !self
            .user_repo
            .exists(&mut txn, user_id)
            .await
            .context("Failed to check user existence")?
        {
            return Err(MfaDisableError::NotFound);
        }

        let totp_devices = self
            .mfa_repo
            .list_totp_devices_by_user(&mut txn, user_id)
            .await
            .context("Failed to get totp devices from database")?;

        if totp_devices.iter().all(|x| !x.enabled) {
            return Err(MfaDisableError::NotEnabled);
        }

        self.mfa_disable
            .disable(&mut txn, user_id)
            .await
            .context("Failed to disable mfa")?;

        txn.commit().await?;

        Ok(())
    }
}
