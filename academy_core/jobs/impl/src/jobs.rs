use academy_core_jobs_contracts::{
    jobs::{JobCreateCommand, JobListResult, JobsService},
    JobCreateError,
};
use academy_di::Build;
use academy_models::job::Job;
use academy_persistence_contracts::jobs::{JobsRepoError, JobsRepository};
use academy_shared_contracts::id::IdService;
use academy_utils::trace_instrument;
use anyhow::{anyhow, Context};

#[derive(Debug, Clone, Copy, Build, Default)]
pub struct JobsServiceImpl<Id, JobsRepo> {
    id: Id,
    jobs_repo: JobsRepo,
}

impl<Txn, Id, JobsRepo> JobsService<Txn> for JobsServiceImpl<Id, JobsRepo>
where
    Txn: Send + Sync + 'static,
    Id: IdService,
    JobsRepo: JobsRepository<Txn>,
{
    #[trace_instrument(skip(self, txn))]
    async fn get_jobs(&self, txn: &mut Txn) -> anyhow::Result<JobListResult> {
        let jobs = self
            .jobs_repo
            .list(txn)
            .await
            .context("Failed to get jobs from database")?;

        let total = u64::try_from(jobs.len()).unwrap();

        Ok(JobListResult { total, jobs })
    }

    #[trace_instrument(skip(self, txn))]
    async fn create_job(
        &self,
        txn: &mut Txn,
        JobCreateCommand { title }: JobCreateCommand,
    ) -> Result<Job, JobCreateError> {
        let job = Job {
            id: self.id.generate(),
            title,
        };
        self.jobs_repo
            .create(txn, &job)
            .await
            .map_err(|err| match err {
                JobsRepoError::Other(err) => {
                    anyhow!(err).context("Failed to create job in database")
                }
            })?;

        Ok(job)
    }
}
