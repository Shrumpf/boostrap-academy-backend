use std::future::Future;

use academy_models::{
    user::{UserId, UserPassword},
    VerificationCode,
};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserResetPasswordCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        code: VerificationCode,
        new_password: UserPassword,
    ) -> impl Future<Output = Result<(), UserResetPasswordCommandError>> + Send;
}

#[derive(Debug, Error)]
pub enum UserResetPasswordCommandError {
    #[error("The verification code is invalid.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserResetPasswordCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        user_id: UserId,
        code: VerificationCode,
        new_password: UserPassword,
        result: Result<(), UserResetPasswordCommandError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(code),
                mockall::predicate::eq(new_password),
            )
            .return_once(|_, _, _, _| Box::pin(std::future::ready(result)));
        self
    }
}
