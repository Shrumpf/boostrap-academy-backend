use std::future::Future;

use academy_models::job::{Job, JobTitle};

use crate::JobCreateError;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait JobsService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn get_jobs(&self, txn: &mut Txn)
        -> impl Future<Output = anyhow::Result<JobListResult>> + Send;

    fn create_job(
        &self,
        txn: &mut Txn,
        cmd: JobCreateCommand,
    ) -> impl Future<Output = Result<Job, JobCreateError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobListResult {
    pub total: u64,
    pub jobs: Vec<Job>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobCreateCommand {
    pub title: JobTitle,
}
