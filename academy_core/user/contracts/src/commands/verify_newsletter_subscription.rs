use std::future::Future;

use academy_models::{user::UserId, VerificationCode};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserVerifyNewsletterSubscriptionCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        code: VerificationCode,
    ) -> impl Future<Output = Result<(), UserVerifyNewsletterSubscriptionCommandError>> + Send;
}

#[derive(Debug, Error)]
pub enum UserVerifyNewsletterSubscriptionCommandError {
    #[error("The verification code is incorrect.")]
    InvalidCode,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserVerifyNewsletterSubscriptionCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        user_id: UserId,
        code: VerificationCode,
        result: Result<(), UserVerifyNewsletterSubscriptionCommandError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(code),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(result)));
        self
    }
}
