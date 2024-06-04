use std::future::Future;

use academy_models::session::SessionRefreshTokenHash;

/// Invalidates a previously issued access token using the corresponding
/// refresh token hash.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait AuthInvalidateAccessTokenCommandService: Send + Sync + 'static {
    fn invoke(
        &self,
        refresh_token_hash: SessionRefreshTokenHash,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl MockAuthInvalidateAccessTokenCommandService {
    pub fn with_invoke(mut self, refresh_token_hash: SessionRefreshTokenHash) -> Self {
        self.expect_invoke()
            .once()
            .with(mockall::predicate::eq(refresh_token_hash))
            .return_once(|_| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
