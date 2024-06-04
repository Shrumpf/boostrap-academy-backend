use academy_core_mfa_contracts::commands::disable::MfaDisableCommandService;
use academy_di::Build;
use academy_models::user::UserId;
use academy_persistence_contracts::mfa::MfaRepository;

#[derive(Debug, Clone, Build)]
pub struct MfaDisableCommandServiceImpl<MfaRepo> {
    mfa_repo: MfaRepo,
}

impl<Txn, MfaRepo> MfaDisableCommandService<Txn> for MfaDisableCommandServiceImpl<MfaRepo>
where
    Txn: Send + Sync + 'static,
    MfaRepo: MfaRepository<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, user_id: UserId) -> anyhow::Result<()> {
        self.mfa_repo
            .delete_totp_devices_by_user(txn, user_id)
            .await?;

        self.mfa_repo
            .delete_mfa_recovery_code_hash(txn, user_id)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::user::FOO;
    use academy_persistence_contracts::mfa::MockMfaRepository;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let mfa_repo = MockMfaRepository::new()
            .with_delete_totp_devices_by_user(FOO.user.id)
            .with_delete_mfa_recovery_code_hash(FOO.user.id);

        let sut = MfaDisableCommandServiceImpl { mfa_repo };

        // Act
        let result = sut.invoke(&mut (), FOO.user.id).await;

        // Assert
        result.unwrap();
    }
}
