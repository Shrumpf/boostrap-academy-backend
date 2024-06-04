use chrono::{DateTime, Utc};

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait TimeService: Send + Sync + 'static {
    /// Returns the current time.
    fn now(&self) -> DateTime<Utc>;
}

#[cfg(feature = "mock")]
impl MockTimeService {
    pub fn with_now(mut self, time: DateTime<Utc>) -> Self {
        self.expect_now().once().return_const(time);
        self
    }
}
