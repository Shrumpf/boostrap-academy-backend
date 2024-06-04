use uuid::Uuid;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait IdService: Send + Sync + 'static {
    /// Generates a new unique ID.
    fn generate<I: From<Uuid> + 'static>(&self) -> I;
}

#[cfg(feature = "mock")]
impl MockIdService {
    pub fn with_generate<I: From<Uuid> + Send + 'static>(mut self, id: I) -> Self {
        self.expect_generate().once().return_once(|| id);
        self
    }
}
