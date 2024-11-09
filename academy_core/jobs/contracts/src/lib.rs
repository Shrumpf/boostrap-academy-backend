use std::future::Future;

use academy_models::job::{Job, JobTitle};
use anyhow::Error;
use jobs::JobListResult;
use thiserror::Error;

pub mod jobs;

pub trait JobsFeatureService: Send + Sync + 'static {
    /// return all jobs
    fn get_jobs(&self) -> impl Future<Output = Result<JobListResult, Error>> + Send;

    fn create_job(
        &self,
        request: JobCreateRequest,
    ) -> impl Future<Output = Result<Job, JobCreateError>> + Send;
}

#[derive(Debug)]
pub struct JobCreateRequest {
    pub title: JobTitle,
}

#[derive(Debug, Error)]
pub enum JobCreateError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
