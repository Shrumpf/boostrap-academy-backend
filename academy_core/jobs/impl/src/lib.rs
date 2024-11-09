use academy_core_jobs_contracts::{
    jobs::{JobCreateCommand, JobListResult, JobsService},
    JobCreateError, JobCreateRequest, JobsFeatureService,
};
use academy_di::Build;
use academy_models::job::Job;
use academy_persistence_contracts::{jobs::JobsRepository, Database, Transaction};
use academy_utils::trace_instrument;
use anyhow::{Context, Error};

pub mod jobs;

#[derive(Debug, Clone)]
pub struct JobsFeatureConfig {}

#[derive(Debug, Clone, Default, Build)]
pub struct JobsFeatureServiceImpl<Db, JobsRepo, JobsServ> {
    db: Db,
    jobs_repo: JobsRepo,
    jobs_service: JobsServ,
}

impl<Db, JobsRepo, JobsServ> JobsFeatureService for JobsFeatureServiceImpl<Db, JobsRepo, JobsServ>
where
    Db: Database,
    JobsRepo: JobsRepository<Db::Transaction>,
    JobsServ: JobsService<Db::Transaction>,
{
    #[trace_instrument(skip(self))]
    async fn get_jobs(&self) -> Result<JobListResult, Error> {
        let mut txn = self.db.begin_transaction().await.unwrap();
        self.jobs_service
            .get_jobs(&mut txn)
            .await
            .context("Failed to list jobs")
            .map_err(Into::into)
    }

    #[trace_instrument(skip(self))]
    async fn create_job(&self, request: JobCreateRequest) -> Result<Job, JobCreateError> {
        let mut txn = self.db.begin_transaction().await.unwrap();
        let cmd = JobCreateCommand {
            title: request.title,
        };

        let result = self
            .jobs_service
            .create_job(&mut txn, cmd)
            .await
            .context("Failed to create job")?;

        txn.commit().await.unwrap();

        Ok(result)
    }
}
