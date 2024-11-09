use std::future::Future;

use academy_models::job::Job;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JobsRepoError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait JobsRepository<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn list(&self, txn: &mut Txn) -> impl Future<Output = anyhow::Result<Vec<Job>>> + Send;
    fn create(
        &self,
        txn: &mut Txn,
        job: &Job,
    ) -> impl Future<Output = Result<(), JobsRepoError>> + Send;
}
