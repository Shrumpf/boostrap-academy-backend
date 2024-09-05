use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait InternalAuthService: Send + Sync + 'static {
    fn authenticate(&self, token: &str, audience: &str) -> Result<(), InternalAuthError>;
}

#[derive(Debug, Error)]
pub enum InternalAuthError {
    #[error("The auth token is invalid.")]
    InvalidToken,
}

#[cfg(feature = "mock")]
impl MockInternalAuthService {
    pub fn with_authenticate(mut self, audience: &'static str, ok: bool) -> Self {
        self.expect_authenticate()
            .once()
            .with(
                mockall::predicate::eq("token"),
                mockall::predicate::eq(audience),
            )
            .return_once(move |_, _| ok.then_some(()).ok_or(InternalAuthError::InvalidToken));
        self
    }
}
