use std::future::Future;

use email_address::EmailAddress;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserRequestVerificationEmailCommandService: Send + Sync + 'static {
    fn invoke(&self, email: EmailAddress) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl MockUserRequestVerificationEmailCommandService {
    pub fn with_invoke(mut self, email: EmailAddress) -> Self {
        self.expect_invoke()
            .once()
            .with(mockall::predicate::eq(email))
            .return_once(|_| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
