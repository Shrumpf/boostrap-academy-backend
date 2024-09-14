use academy_models::session::SessionRefreshTokenHash;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait AuthRefreshTokenService: Send + Sync + 'static {
    fn issue(&self) -> (String, SessionRefreshTokenHash);

    fn hash(&self, refresh_token: &str) -> SessionRefreshTokenHash;
}

#[cfg(feature = "mock")]
impl MockAuthRefreshTokenService {
    pub fn with_issue(
        mut self,
        refresh_token: String,
        refresh_token_hash: SessionRefreshTokenHash,
    ) -> Self {
        self.expect_issue()
            .once()
            .with()
            .return_once(move || (refresh_token, refresh_token_hash));
        self
    }

    pub fn with_hash(
        mut self,
        refresh_token: String,
        refresh_token_hash: SessionRefreshTokenHash,
    ) -> Self {
        self.expect_hash()
            .once()
            .with(mockall::predicate::eq(refresh_token))
            .return_once(move |_| refresh_token_hash);
        self
    }
}
