use std::sync::Arc;

use academy_core_jobs_contracts::{
    jobs::JobListResult, JobCreateError, JobCreateRequest, JobsFeatureService,
};
use academy_models::job::{Job, JobTitle};
use aide::{
    axum::{routing, ApiRouter},
    transform::TransformOperation,
};
use anyhow::Error;
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    docs::TransformOperationExt,
    errors::{internal_server_error, internal_server_error_docs},
    models::job::ApiJob,
};

pub const TAG: &str = "Jobs";

pub fn router(service: Arc<impl JobsFeatureService>) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route(
            "/jobs/jobs",
            routing::get_with(jobs, jobs_docs).post_with(create, create_docs),
        )
        .with_state(service)
        .with_path_items(|op| op.tag(TAG))
}

#[derive(Serialize, JsonSchema)]
struct ListResult {
    /// The total number of users matching the given query
    total: u64,
    /// The paginated list of users matching the given query
    jobs: Vec<ApiJob>,
}

async fn jobs(jobs_service: State<Arc<impl JobsFeatureService>>) -> Response {
    match jobs_service.get_jobs().await {
        Ok(JobListResult { total, jobs }) => {
            Json(jobs.into_iter().map(ApiJob::from).collect::<Vec<ApiJob>>()).into_response()
        }
        Err(err) => internal_server_error(err),
    }
}

fn jobs_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Foo Jobs")
        .add_response::<ListResult>(StatusCode::OK, None)
        .with(internal_server_error_docs)
}

#[derive(Deserialize, JsonSchema)]
struct CreateRequest {
    title: JobTitle,
}

async fn create(
    jobs_service: State<Arc<impl JobsFeatureService>>,
    Json(CreateRequest { title }): Json<CreateRequest>,
) -> Response {
    match jobs_service.create_job(JobCreateRequest { title }).await {
        Ok(result) => Json(ApiJob::from(result)).into_response(),
        Err(JobCreateError::Other(err)) => internal_server_error(err),
    }
}

fn create_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Create a new job")
        .add_response::<ApiJob>(StatusCode::OK, None)
        .with(internal_server_error_docs)
}
