use std::future::Future;

use academy_models::{mfa::MfaAuthentication, user::UserId};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaAuthenticateService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    /// Authenticate the given user using a second factor.
    ///
    /// Disables MFA for the user if a correct recovery code is provided.
    fn authenticate(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        cmd: MfaAuthentication,
    ) -> impl Future<Output = Result<MfaAuthenticateResult, MfaAuthenticateError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MfaAuthenticateResult {
    /// MFA is disabled
    Disabled,
    /// MFA is enabled and authentication was successful
    Ok,
    /// Recovery code has been used and MFA has been disabled
    Reset,
}

#[derive(Debug, Error)]
pub enum MfaAuthenticateError {
    #[error("The user failed to authenticate.")]
    Failed,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockMfaAuthenticateService<Txn> {
    pub fn with_authenticate(
        mut self,
        user_id: UserId,
        cmd: MfaAuthentication,
        result: Result<MfaAuthenticateResult, MfaAuthenticateError>,
    ) -> Self {
        self.expect_authenticate()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(cmd),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(result)));
        self
    }
}
