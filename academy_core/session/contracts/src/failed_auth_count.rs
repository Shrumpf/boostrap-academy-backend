use std::future::Future;

use academy_models::user::UserNameOrEmailAddress;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait SessionFailedAuthCountService: Send + Sync + 'static {
    /// Return the number of failed authentication attempts for the given login.
    fn get(
        &self,
        name_or_email: &UserNameOrEmailAddress,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;

    /// Increment the number of failed authentication attempts for the given
    /// login.
    fn increment(
        &self,
        name_or_email: &UserNameOrEmailAddress,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Reset the number of failed authentication attempts for the given login.
    fn reset(
        &self,
        name_or_email: &UserNameOrEmailAddress,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl MockSessionFailedAuthCountService {
    pub fn with_get(mut self, name_or_email: UserNameOrEmailAddress, result: u64) -> Self {
        self.expect_get()
            .once()
            .with(mockall::predicate::eq(name_or_email))
            .return_once(move |_| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_increment(mut self, name_or_email: UserNameOrEmailAddress) -> Self {
        self.expect_increment()
            .once()
            .with(mockall::predicate::eq(name_or_email))
            .return_once(|_| Box::pin(std::future::ready(Ok(()))));
        self
    }

    pub fn with_reset(mut self, name_or_email: UserNameOrEmailAddress) -> Self {
        self.expect_reset()
            .once()
            .with(mockall::predicate::eq(name_or_email))
            .return_once(|_| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
