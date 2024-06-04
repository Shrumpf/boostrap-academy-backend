use std::future::Future;

use academy_models::{
    pagination::PaginationSlice,
    user::{UserComposite, UserFilter},
};

/// Returns all users matching the given query.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserListQueryService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        query: UserListQuery,
    ) -> impl Future<Output = anyhow::Result<UserListResult>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserListQuery {
    pub pagination: PaginationSlice,
    pub filter: UserFilter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserListResult {
    pub total: u64,
    pub user_composites: Vec<UserComposite>,
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserListQueryService<Txn> {
    pub fn with_invoke(mut self, query: UserListQuery, result: UserListResult) -> Self {
        self.expect_invoke()
            .once()
            .with(mockall::predicate::always(), mockall::predicate::eq(query))
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
