use std::future::Future;

use academy_models::user::UserId;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait InternalApiService: Send + Sync + 'static {
    fn release_coins(&self, user_id: UserId) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl MockInternalApiService {
    pub fn with_release_coins(mut self, user_id: UserId) -> Self {
        self.expect_release_coins()
            .once()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
