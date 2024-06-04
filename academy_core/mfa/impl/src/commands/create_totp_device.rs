use academy_core_mfa_contracts::commands::create_totp_device::MfaCreateTotpDeviceCommandService;
use academy_di::Build;
use academy_models::{
    mfa::{TotpDevice, TotpSetup},
    user::UserId,
};
use academy_persistence_contracts::mfa::MfaRepository;
use academy_shared_contracts::{id::IdService, time::TimeService, totp::TotpService};

#[derive(Debug, Clone, Build)]
pub struct MfaCreateTotpDeviceCommandServiceImpl<Id, Time, Totp, MfaRepo> {
    id: Id,
    time: Time,
    totp: Totp,
    mfa_repo: MfaRepo,
}

impl<Txn, Id, Time, Totp, MfaRepo> MfaCreateTotpDeviceCommandService<Txn>
    for MfaCreateTotpDeviceCommandServiceImpl<Id, Time, Totp, MfaRepo>
where
    Txn: Send + Sync + 'static,
    Id: IdService,
    Time: TimeService,
    Totp: TotpService,
    MfaRepo: MfaRepository<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, user_id: UserId) -> anyhow::Result<TotpSetup> {
        let (secret, setup) = self.totp.generate_secret();

        let totp_device = TotpDevice {
            id: self.id.generate(),
            user_id,
            enabled: false,
            created_at: self.time.now(),
        };

        self.mfa_repo
            .create_totp_device(txn, &totp_device, &secret)
            .await?;

        Ok(setup)
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::{mfa::FOO_TOTP_1, user::FOO};
    use academy_models::mfa::TotpSecret;
    use academy_persistence_contracts::mfa::MockMfaRepository;
    use academy_shared_contracts::{
        id::MockIdService, time::MockTimeService, totp::MockTotpService,
    };

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let secret = TotpSecret::try_new("the random totp secret".to_owned().into_bytes()).unwrap();
        let setup = TotpSetup {
            secret: "ORUGKIDSMFXGI33NEB2G65DQEBZWKY3SMV2A".into(),
        };

        let id = MockIdService::new().with_generate(FOO_TOTP_1.id);
        let time = MockTimeService::new().with_now(FOO_TOTP_1.created_at);

        let totp = MockTotpService::new().with_generate_secret(secret.clone(), setup.clone());

        let mfa_repo = MockMfaRepository::new().with_create_totp_device(FOO_TOTP_1.clone(), secret);

        let sut = MfaCreateTotpDeviceCommandServiceImpl {
            id,
            time,
            totp,
            mfa_repo,
        };

        // Act
        let result = sut.invoke(&mut (), FOO.user.id).await;

        // Assert
        assert_eq!(result.unwrap(), setup);
    }
}
