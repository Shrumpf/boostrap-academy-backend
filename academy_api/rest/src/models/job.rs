use academy_models::job::{Job, JobId, JobTitle};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::const_schema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApiJob {
    pub id: JobId,
    pub title: JobTitle,
}

impl From<Job> for ApiJob {
    fn from(job: Job) -> Self {
        let Job { id, title } = job;

        Self { id, title }
    }
}
