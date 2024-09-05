use std::future::Future;

use academy_models::{email_address::EmailAddress, user::UserId};
use thiserror::Error;

/// Updates a user's email address and it's verification status.
///
/// Also invalidates any access token associated with this user.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserUpdateEmailCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        email: &Option<EmailAddress>,
        email_verified: bool,
    ) -> impl Future<Output = Result<bool, UserUpdateEmailCommandError>> + Send;
}

#[derive(Debug, Error)]
pub enum UserUpdateEmailCommandError {
    #[error("A user with the same email address already exists.")]
    Conflict,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserUpdateEmailCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        user_id: UserId,
        email: EmailAddress,
        email_verified: bool,
        result: Result<bool, UserUpdateEmailCommandError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(Some(email)),
                mockall::predicate::eq(email_verified),
            )
            .return_once(|_, _, _, _| Box::pin(std::future::ready(result)));
        self
    }
}
