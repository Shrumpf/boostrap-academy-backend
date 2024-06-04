use std::future::Future;

use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait PasswordService: Send + Sync + 'static {
    /// Securely hashes a password.
    fn hash(&self, password: String) -> impl Future<Output = anyhow::Result<String>> + Send;

    /// Verifies that a password matches the given hash.
    fn verify(
        &self,
        password: String,
        hash: String,
    ) -> impl Future<Output = Result<(), PasswordVerifyError>> + Send;
}

#[derive(Debug, Error)]
pub enum PasswordVerifyError {
    #[error("The password does not match the provided hash.")]
    InvalidPassword,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl MockPasswordService {
    pub fn with_hash(mut self, password: String, hash: String) -> Self {
        self.expect_hash()
            .once()
            .with(mockall::predicate::eq(password))
            .return_once(|_| Box::pin(std::future::ready(Ok(hash))));
        self
    }

    pub fn with_verify(mut self, password: String, hash: String, ok: bool) -> Self {
        let result = ok.then_some(()).ok_or(PasswordVerifyError::InvalidPassword);
        self.expect_verify()
            .once()
            .with(
                mockall::predicate::eq(password),
                mockall::predicate::eq(hash),
            )
            .return_once(|_, _| Box::pin(std::future::ready(result)));
        self
    }
}
