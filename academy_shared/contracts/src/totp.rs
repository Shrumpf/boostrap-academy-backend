use std::future::Future;

use academy_models::mfa::{TotpCode, TotpSecret, TotpSetup};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait TotpService: Send + Sync + 'static {
    /// Generate a new random totp secret.
    fn generate_secret(&self) -> (TotpSecret, TotpSetup);

    /// Check the given totp code.
    fn check(
        &self,
        code: &TotpCode,
        secret: TotpSecret,
    ) -> impl Future<Output = Result<(), TotpCheckError>> + Send;
}

#[derive(Debug, Error)]
pub enum TotpCheckError {
    #[error("The code is incorrect.")]
    InvalidCode,
    #[error("The code has already been used recently.")]
    RecentlyUsed,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl MockTotpService {
    pub fn with_generate_secret(mut self, secret: TotpSecret, setup: TotpSetup) -> Self {
        self.expect_generate_secret()
            .once()
            .with()
            .return_once(|| (secret, setup));
        self
    }

    pub fn with_check(
        mut self,
        code: TotpCode,
        secret: TotpSecret,
        result: Result<(), TotpCheckError>,
    ) -> Self {
        self.expect_check()
            .once()
            .with(mockall::predicate::eq(code), mockall::predicate::eq(secret))
            .return_once(|_, _| Box::pin(std::future::ready(result)));
        self
    }
}
