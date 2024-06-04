use std::future::Future;

use academy_models::{mfa::MfaRecoveryCode, user::UserId};

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaSetupRecoveryCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<MfaRecoveryCode>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockMfaSetupRecoveryCommandService<Txn> {
    pub fn with_invoke(mut self, user_id: UserId, recovery_code: MfaRecoveryCode) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(recovery_code))));
        self
    }
}
