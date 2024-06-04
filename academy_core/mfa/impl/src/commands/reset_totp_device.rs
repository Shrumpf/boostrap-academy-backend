use academy_core_mfa_contracts::commands::reset_totp_device::MfaResetTotpDeviceCommandService;
use academy_di::Build;
use academy_models::mfa::{TotpDeviceId, TotpDevicePatchRef, TotpSetup};
use academy_persistence_contracts::mfa::MfaRepository;
use academy_shared_contracts::totp::TotpService;

#[derive(Debug, Clone, Build)]
pub struct MfaResetTotpDeviceCommandServiceImpl<Totp, MfaRepo> {
    totp: Totp,
    mfa_repo: MfaRepo,
}

impl<Txn, Totp, MfaRepo> MfaResetTotpDeviceCommandService<Txn>
    for MfaResetTotpDeviceCommandServiceImpl<Totp, MfaRepo>
where
    Txn: Send + Sync + 'static,
    Totp: TotpService,
    MfaRepo: MfaRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        totp_device_id: TotpDeviceId,
    ) -> anyhow::Result<TotpSetup> {
        let (secret, setup) = self.totp.generate_secret();

        self.mfa_repo
            .update_totp_device(
                txn,
                totp_device_id,
                TotpDevicePatchRef::new().update_enabled(&false),
            )
            .await?;

        self.mfa_repo
            .save_totp_device_secret(txn, totp_device_id, &secret)
            .await?;

        Ok(setup)
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::mfa::FOO_TOTP_1;
    use academy_models::mfa::{TotpDevicePatch, TotpSecret};
    use academy_persistence_contracts::mfa::MockMfaRepository;
    use academy_shared_contracts::totp::MockTotpService;

    use super::*;
    #[tokio::test]
    async fn ok() {
        // Arrange
        let secret = TotpSecret::try_new("the random totp secret".to_owned().into_bytes()).unwrap();
        let setup = TotpSetup {
            secret: "ORUGKIDSMFXGI33NEB2G65DQEBZWKY3SMV2A".into(),
        };

        let totp = MockTotpService::new().with_generate_secret(secret.clone(), setup.clone());

        let mfa_repo = MockMfaRepository::new()
            .with_update_totp_device(
                FOO_TOTP_1.id,
                TotpDevicePatch::new().update_enabled(false),
                true,
            )
            .with_save_totp_device_secret(FOO_TOTP_1.id, secret);

        let sut = MfaResetTotpDeviceCommandServiceImpl { totp, mfa_repo };

        // Act
        let result = sut.invoke(&mut (), FOO_TOTP_1.id).await;

        // Assert
        assert_eq!(result.unwrap(), setup);
    }
}
