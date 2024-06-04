use std::future::Future;

use academy_models::user::UserId;
use email_address::EmailAddress;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserRequestPasswordResetEmailCommandService: Send + Sync + 'static {
    fn invoke(
        &self,
        user_id: UserId,
        email: EmailAddress,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl MockUserRequestPasswordResetEmailCommandService {
    pub fn with_invoke(mut self, user_id: UserId, email: EmailAddress) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(email),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
