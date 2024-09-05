use std::future::Future;

use academy_models::{email_address::EmailAddressWithName, user::UserId};

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserRequestSubscribeNewsletterEmailCommandService: Send + Sync + 'static {
    fn invoke(
        &self,
        user_id: UserId,
        email: EmailAddressWithName,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl MockUserRequestSubscribeNewsletterEmailCommandService {
    pub fn with_invoke(mut self, user_id: UserId, email: EmailAddressWithName) -> Self {
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
