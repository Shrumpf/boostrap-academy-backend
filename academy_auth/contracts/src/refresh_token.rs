use academy_models::{auth::RefreshToken, session::SessionRefreshTokenHash};

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait AuthRefreshTokenService: Send + Sync + 'static {
    /// Generate a new refresh token.
    fn issue(&self) -> RefreshToken;

    /// Return the hash of the given refresh token.
    fn hash(&self, refresh_token: &RefreshToken) -> SessionRefreshTokenHash;
}

#[cfg(feature = "mock")]
impl MockAuthRefreshTokenService {
    pub fn with_issue(mut self, refresh_token: RefreshToken) -> Self {
        self.expect_issue()
            .once()
            .with()
            .return_once(move || refresh_token);
        self
    }

    pub fn with_hash(
        mut self,
        refresh_token: RefreshToken,
        refresh_token_hash: SessionRefreshTokenHash,
    ) -> Self {
        self.expect_hash()
            .once()
            .with(mockall::predicate::eq(refresh_token))
            .return_once(move |_| refresh_token_hash);
        self
    }
}
