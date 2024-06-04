use academy_models::pagination::{PaginationLimit, PaginationSlice};
use serde::{Deserialize, Serialize};

pub mod contact;
pub mod session;
pub mod user;

#[derive(Serialize)]
pub struct ApiError {
    pub detail: &'static str,
}

#[derive(Deserialize)]
pub struct ApiPaginationSlice {
    #[serde(default)]
    pub limit: PaginationLimit,
    #[serde(default)]
    pub offset: u64,
}

impl From<ApiPaginationSlice> for PaginationSlice {
    fn from(value: ApiPaginationSlice) -> Self {
        Self {
            limit: value.limit,
            offset: value.offset,
        }
    }
}
