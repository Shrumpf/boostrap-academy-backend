use std::future::Future;

use academy_models::{mfa::TotpSetup, user::UserId};

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaCreateTotpDeviceCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<TotpSetup>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockMfaCreateTotpDeviceCommandService<Txn> {
    pub fn with_invoke(mut self, user_id: UserId, result: TotpSetup) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
