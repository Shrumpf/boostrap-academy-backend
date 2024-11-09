use academy_di::Build;
use academy_models::job::Job;
use academy_persistence_contracts::jobs::{JobsRepoError, JobsRepository};
use academy_utils::trace_instrument;
use bb8_postgres::tokio_postgres::{self, Row};
use uuid::Uuid;

use crate::{arg_indices, columns, ColumnCounter, PostgresTransaction};

#[derive(Debug, Clone, Copy, Default, Build)]
pub struct PostgresJobsRepository;

columns!(jobs as "j": "id", "title");

impl JobsRepository<PostgresTransaction> for PostgresJobsRepository {
    #[trace_instrument(skip(self, txn))]
    async fn list(&self, txn: &mut PostgresTransaction) -> anyhow::Result<Vec<Job>> {
        let query = format!("SELECT {JOBS_COLS} FROM jobs j where true").to_owned();
        println!("{}", query);
        txn.txn()
            .query(&query, &[])
            .await
            .map_err(Into::into)
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| decode_jobs(&row, &mut Default::default()))
                    .collect()
            })
    }

    #[trace_instrument(skip(self, txn))]
    async fn create(&self, txn: &mut PostgresTransaction, job: &Job) -> Result<(), JobsRepoError> {
        println!("{:?}", job);
        txn.txn()
            .execute(
                &format!(
                    "INSERT INTO jobs ({JOBS_COL_NAMES}) VALUES ({})",
                    arg_indices(1..=JOBS_CNT)
                ),
                &[&*job.id, &job.title.as_str()],
            )
            .await
            .map_err(map_jobs_repo_error)?;

        Ok(())
    }
}

fn decode_jobs(row: &Row, cnt: &mut ColumnCounter) -> anyhow::Result<Job> {
    Ok(Job {
        id: row.get::<_, Uuid>(cnt.idx()).into(),
        title: row.get::<_, String>(cnt.idx()).try_into()?,
    })
}

fn map_jobs_repo_error(err: tokio_postgres::Error) -> JobsRepoError {
    err.as_db_error();
    JobsRepoError::Other(err.into())
}
