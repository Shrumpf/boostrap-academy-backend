use academy_utils::patch::Patch;

use crate::macros::{id, nutype_string};

id!(JobId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JobIdOrSelf {
    JobId(JobId),
    Slf,
}

impl From<JobId> for JobIdOrSelf {
    fn from(value: JobId) -> Self {
        Self::JobId(value)
    }
}

impl JobIdOrSelf {
    pub fn unwrap_or(self, self_job_id: JobId) -> JobId {
        match self {
            JobIdOrSelf::JobId(job_id) => job_id,
            JobIdOrSelf::Slf => self_job_id,
        }
    }
}

nutype_string!(JobTitle(validate(len_char_min = 1, len_char_max = 128)));

#[derive(Debug, Clone, PartialEq, Eq, Patch)]
pub struct Job {
    #[no_patch]
    pub id: JobId,
    pub title: JobTitle,
}
