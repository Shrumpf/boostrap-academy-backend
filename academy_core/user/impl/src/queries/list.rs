use academy_core_user_contracts::queries::list::{
    UserListQuery, UserListQueryService, UserListResult,
};
use academy_di::Build;
use academy_persistence_contracts::user::UserRepository;

#[derive(Debug, Clone, Build)]
pub struct UserListQueryServiceImpl<UserRepo> {
    user_repo: UserRepo,
}

impl<Txn, UserRepo> UserListQueryService<Txn> for UserListQueryServiceImpl<UserRepo>
where
    Txn: Send + Sync + 'static,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, query: UserListQuery) -> anyhow::Result<UserListResult> {
        let total = self.user_repo.count(txn, &query.filter).await?;
        let user_composites = self
            .user_repo
            .list_composites(txn, &query.filter, query.pagination)
            .await?;
        Ok(UserListResult {
            total,
            user_composites,
        })
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::user::ALL_USERS;
    use academy_models::{pagination::PaginationSlice, user::UserFilter};
    use academy_persistence_contracts::user::MockUserRepository;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let query = UserListQuery {
            pagination: PaginationSlice {
                limit: 42.try_into().unwrap(),
                offset: 7,
            },
            filter: UserFilter {
                name: Some("the name".try_into().unwrap()),
                email: Some("the email".try_into().unwrap()),
                enabled: Some(true),
                admin: Some(false),
                mfa_enabled: None,
                email_verified: Some(true),
                newsletter: Some(false),
            },
        };
        let expected = ALL_USERS.iter().copied().cloned().collect::<Vec<_>>();

        let user_repo = MockUserRepository::new()
            .with_count(query.filter.clone(), 17)
            .with_list_composites(query.filter.clone(), query.pagination, expected.clone());

        let sut = UserListQueryServiceImpl { user_repo };

        // Act
        let result = sut.invoke(&mut (), query).await;

        // Assert
        let result = result.unwrap();
        assert_eq!(result.user_composites, expected);
    }
}
