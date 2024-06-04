use academy_core_mfa_contracts::commands::confirm_totp_device::{
    MfaConfirmTotpDeviceCommandError, MfaConfirmTotpDeviceCommandService,
};
use academy_di::Build;
use academy_models::mfa::{TotpCode, TotpDevice, TotpDevicePatch};
use academy_persistence_contracts::mfa::MfaRepository;
use academy_shared_contracts::totp::{TotpCheckError, TotpService};
use academy_utils::patch::Patch;

#[derive(Debug, Clone, Build)]
pub struct MfaConfirmTotpDeviceCommandServiceImpl<Totp, MfaRepo> {
    totp: Totp,
    mfa_repo: MfaRepo,
}

impl<Txn, Totp, MfaRepo> MfaConfirmTotpDeviceCommandService<Txn>
    for MfaConfirmTotpDeviceCommandServiceImpl<Totp, MfaRepo>
where
    Txn: Send + Sync + 'static,
    Totp: TotpService,
    MfaRepo: MfaRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        totp_device: TotpDevice,
        code: TotpCode,
    ) -> Result<TotpDevice, MfaConfirmTotpDeviceCommandError> {
        let secret = self
            .mfa_repo
            .get_totp_device_secret(txn, totp_device.id)
            .await?;

        self.totp
            .check(&code, secret)
            .await
            .map_err(|err| match err {
                TotpCheckError::InvalidCode | TotpCheckError::RecentlyUsed => {
                    MfaConfirmTotpDeviceCommandError::InvalidCode
                }
                TotpCheckError::Other(err) => err.into(),
            })?;

        let patch = TotpDevicePatch::new().update_enabled(true);
        self.mfa_repo
            .update_totp_device(txn, totp_device.id, patch.as_ref())
            .await?;

        Ok(totp_device.update(patch))
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::mfa::FOO_TOTP_1;
    use academy_models::mfa::TotpSecret;
    use academy_persistence_contracts::mfa::MockMfaRepository;
    use academy_shared_contracts::totp::MockTotpService;
    use academy_utils::{assert_matches, Apply};

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let expected = FOO_TOTP_1.clone().with(|x| x.enabled = true);
        let code = TotpCode::try_new("123456").unwrap();
        let secret = TotpSecret::try_new("the random totp secret".to_owned().into_bytes()).unwrap();

        let totp = MockTotpService::new().with_check(code.clone(), secret.clone(), Ok(()));

        let mfa_repo = MockMfaRepository::new()
            .with_get_totp_device_secret(FOO_TOTP_1.id, secret)
            .with_update_totp_device(
                FOO_TOTP_1.id,
                TotpDevicePatch::new().update_enabled(true),
                true,
            );

        let sut = MfaConfirmTotpDeviceCommandServiceImpl { totp, mfa_repo };

        // Act
        let result = sut.invoke(&mut (), FOO_TOTP_1.clone(), code).await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn invalid_code() {
        // Arrange
        let code = TotpCode::try_new("123456").unwrap();
        let secret = TotpSecret::try_new("the random totp secret".to_owned().into_bytes()).unwrap();

        let totp = MockTotpService::new().with_check(
            code.clone(),
            secret.clone(),
            Err(TotpCheckError::InvalidCode),
        );

        let mfa_repo = MockMfaRepository::new().with_get_totp_device_secret(FOO_TOTP_1.id, secret);

        let sut = MfaConfirmTotpDeviceCommandServiceImpl { totp, mfa_repo };

        // Act
        let result = sut.invoke(&mut (), FOO_TOTP_1.clone(), code).await;

        // Assert
        assert_matches!(result, Err(MfaConfirmTotpDeviceCommandError::InvalidCode));
    }
}
