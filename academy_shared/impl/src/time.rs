use academy_di::Build;
use academy_shared_contracts::time::TimeService;
use academy_utils::trace_instrument;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Build)]
pub struct TimeServiceImpl;

impl TimeService for TimeServiceImpl {
    #[trace_instrument(skip(self))]
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
