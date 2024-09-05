use std::future::Future;

use academy_models::email_address::EmailAddressWithName;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserRequestVerificationEmailCommandService: Send + Sync + 'static {
    fn invoke(
        &self,
        email: EmailAddressWithName,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl MockUserRequestVerificationEmailCommandService {
    pub fn with_invoke(mut self, email: EmailAddressWithName) -> Self {
        self.expect_invoke()
            .once()
            .with(mockall::predicate::eq(email))
            .return_once(|_| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
