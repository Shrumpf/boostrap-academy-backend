use std::future::Future;

use academy_models::{user::UserComposite, VerificationCode};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserVerifyEmailCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        verification_code: &VerificationCode,
    ) -> impl Future<Output = Result<UserComposite, UserVerifyEmailCommandError>> + Send;
}

#[derive(Debug, Error)]
pub enum UserVerifyEmailCommandError {
    #[error("The verification code is invalid.")]
    InvalidCode,
    #[error("The user's email address has already been verified.")]
    AlreadyVerified,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserVerifyEmailCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        verification_code: VerificationCode,
        result: Result<UserComposite, UserVerifyEmailCommandError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(verification_code),
            )
            .return_once(|_, _| Box::pin(std::future::ready(result)));
        self
    }
}
