use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait AuthInternalService: Send + Sync + 'static {
    /// Generate a new internal authentication token.
    fn issue_token(&self, audience: &str) -> anyhow::Result<String>;

    /// Verify an internal authentication token.
    fn authenticate(
        &self,
        token: &str,
        audience: &str,
    ) -> Result<(), AuthInternalAuthenticateError>;
}

#[derive(Debug, Error)]
pub enum AuthInternalAuthenticateError {
    #[error("The auth token is invalid.")]
    InvalidToken,
}

#[cfg(feature = "mock")]
impl MockAuthInternalService {
    pub fn with_issue_token(mut self, audience: &'static str, token: String) -> Self {
        self.expect_issue_token()
            .once()
            .with(mockall::predicate::eq(audience))
            .return_once(|_| Ok(token));
        self
    }

    pub fn with_authenticate(mut self, audience: &'static str, ok: bool) -> Self {
        self.expect_authenticate()
            .once()
            .with(
                mockall::predicate::eq("internal token"),
                mockall::predicate::eq(audience),
            )
            .return_once(move |_, _| {
                ok.then_some(())
                    .ok_or(AuthInternalAuthenticateError::InvalidToken)
            });
        self
    }
}
