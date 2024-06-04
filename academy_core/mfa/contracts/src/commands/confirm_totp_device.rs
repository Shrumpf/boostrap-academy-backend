use std::future::Future;

use academy_models::mfa::{TotpCode, TotpDevice};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaConfirmTotpDeviceCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        totp_device: TotpDevice,
        code: TotpCode,
    ) -> impl Future<Output = Result<TotpDevice, MfaConfirmTotpDeviceCommandError>> + Send;
}

#[derive(Debug, Error)]
pub enum MfaConfirmTotpDeviceCommandError {
    #[error("The totp code is incorrect.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockMfaConfirmTotpDeviceCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        totp_device: TotpDevice,
        code: TotpCode,
        result: Result<TotpDevice, MfaConfirmTotpDeviceCommandError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(totp_device),
                mockall::predicate::eq(code),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(result)));
        self
    }
}
