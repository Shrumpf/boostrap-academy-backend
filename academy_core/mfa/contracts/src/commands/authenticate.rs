use std::future::Future;

use academy_models::{mfa::MfaAuthenticateCommand, user::UserId};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaAuthenticateCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        cmd: MfaAuthenticateCommand,
    ) -> impl Future<Output = Result<MfaAuthenticateCommandResult, MfaAuthenticateCommandError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MfaAuthenticateCommandResult {
    Ok,
    Reset,
}

#[derive(Debug, Error)]
pub enum MfaAuthenticateCommandError {
    #[error("The user failed to authenticate.")]
    Failed,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockMfaAuthenticateCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        user_id: UserId,
        cmd: MfaAuthenticateCommand,
        result: Result<MfaAuthenticateCommandResult, MfaAuthenticateCommandError>,
    ) -> Self {
        self.expect_invoke()
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
