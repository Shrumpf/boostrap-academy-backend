use academy_models::Sha256Hash;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait HashService: Send + Sync + 'static {
    /// Hashes the given data using the SHA-256 algorithm.
    fn sha256(&self, data: &[u8]) -> Sha256Hash;
}

#[cfg(feature = "mock")]
impl MockHashService {
    pub fn with_sha256(mut self, data: Vec<u8>, result: Sha256Hash) -> Self {
        self.expect_sha256()
            .once()
            .with(mockall::predicate::eq(data))
            .return_once(move |_| result);
        self
    }
}
