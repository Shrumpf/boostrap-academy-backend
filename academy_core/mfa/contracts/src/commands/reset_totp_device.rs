use std::future::Future;

use academy_models::mfa::{TotpDeviceId, TotpSetup};

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaResetTotpDeviceCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        totp_device_id: TotpDeviceId,
    ) -> impl Future<Output = anyhow::Result<TotpSetup>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockMfaResetTotpDeviceCommandService<Txn> {
    pub fn with_invoke(mut self, totp_device_id: TotpDeviceId, result: TotpSetup) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(totp_device_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
