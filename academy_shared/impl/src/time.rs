use academy_di::Build;
use academy_shared_contracts::time::TimeService;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Build)]
pub struct TimeServiceImpl;

impl TimeService for TimeServiceImpl {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
