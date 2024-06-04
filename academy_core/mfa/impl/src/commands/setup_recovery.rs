use academy_core_mfa_contracts::commands::setup_recovery::MfaSetupRecoveryCommandService;
use academy_di::Build;
use academy_models::{mfa::MfaRecoveryCode, user::UserId};
use academy_persistence_contracts::mfa::MfaRepository;
use academy_shared_contracts::{hash::HashService, secret::SecretService};

#[derive(Debug, Clone, Build)]
pub struct MfaSetupRecoveryCommandServiceImpl<Secret, Hash, MfaRepo> {
    secret: Secret,
    hash: Hash,
    mfa_repo: MfaRepo,
}

impl<Txn, Secret, Hash, MfaRepo> MfaSetupRecoveryCommandService<Txn>
    for MfaSetupRecoveryCommandServiceImpl<Secret, Hash, MfaRepo>
where
    Txn: Send + Sync + 'static,
    Secret: SecretService,
    Hash: HashService,
    MfaRepo: MfaRepository<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, user_id: UserId) -> anyhow::Result<MfaRecoveryCode> {
        let recovery_code = self.secret.generate_mfa_recovery_code();

        let hash = self.hash.sha256(recovery_code.as_bytes()).into();
        self.mfa_repo
            .save_mfa_recovery_code_hash(txn, user_id, hash)
            .await?;

        Ok(recovery_code)
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::{user::FOO, SHA256HASH1};
    use academy_persistence_contracts::mfa::MockMfaRepository;
    use academy_shared_contracts::{hash::MockHashService, secret::MockSecretService};

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let expected = MfaRecoveryCode::try_new("PJVURV-QRK3YJ-O3U7T6-D50KAC").unwrap();

        let secret = MockSecretService::new().with_generate_mfa_recovery_code(expected.clone());

        let hash = MockHashService::new()
            .with_sha256(expected.clone().into_inner().into_bytes(), *SHA256HASH1);

        let mfa_repo = MockMfaRepository::new()
            .with_save_mfa_recovery_code_hash(FOO.user.id, (*SHA256HASH1).into());

        let sut = MfaSetupRecoveryCommandServiceImpl {
            secret,
            hash,
            mfa_repo,
        };

        // Act
        let result = sut.invoke(&mut (), FOO.user.id).await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }
}
