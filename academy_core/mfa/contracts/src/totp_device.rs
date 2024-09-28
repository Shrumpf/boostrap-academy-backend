use std::future::Future;

use academy_models::{
    mfa::{TotpCode, TotpDevice, TotpDeviceId, TotpSetup},
    user::UserId,
};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaTotpDeviceService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    /// Create a new unconfirmed TOTP device.
    fn create(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<TotpSetup>> + Send;

    /// Confirm a previously created TOTP device.
    fn confirm(
        &self,
        txn: &mut Txn,
        totp_device: TotpDevice,
        code: TotpCode,
    ) -> impl Future<Output = Result<TotpDevice, MfaTotpDeviceConfirmError>> + Send;

    /// Reset an existing TOTP device.
    fn reset(
        &self,
        txn: &mut Txn,
        totp_device_id: TotpDeviceId,
    ) -> impl Future<Output = anyhow::Result<TotpSetup>> + Send;
}

#[derive(Debug, Error)]
pub enum MfaTotpDeviceConfirmError {
    #[error("The totp code is incorrect.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockMfaTotpDeviceService<Txn> {
    pub fn with_create(mut self, user_id: UserId, result: TotpSetup) -> Self {
        self.expect_create()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_confirm(
        mut self,
        totp_device: TotpDevice,
        code: TotpCode,
        result: Result<TotpDevice, MfaTotpDeviceConfirmError>,
    ) -> Self {
        self.expect_confirm()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(totp_device),
                mockall::predicate::eq(code),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(result)));
        self
    }

    pub fn with_reset(mut self, totp_device_id: TotpDeviceId, result: TotpSetup) -> Self {
        self.expect_reset()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(totp_device_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
