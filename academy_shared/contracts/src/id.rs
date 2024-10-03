use std::fmt::Debug;

use uuid::Uuid;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait IdService: Send + Sync + 'static {
    /// Generate a new unique ID.
    fn generate<I: From<Uuid> + Debug + 'static>(&self) -> I;
}

#[cfg(feature = "mock")]
impl MockIdService {
    pub fn with_generate<I: From<Uuid> + Debug + Send + 'static>(mut self, id: I) -> Self {
        self.expect_generate().once().return_once(|| id);
        self
    }
}
