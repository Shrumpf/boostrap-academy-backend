use academy_auth_contracts::AuthService;
use academy_core_session_contracts::commands::delete_by_user::SessionDeleteByUserCommandService;
use academy_di::Build;
use academy_models::user::UserId;
use academy_persistence_contracts::session::SessionRepository;

#[derive(Debug, Clone, Build)]
pub struct SessionDeleteByUserCommandServiceImpl<Auth, SessionRepo> {
    auth: Auth,
    session_repo: SessionRepo,
}

impl<Txn, Auth, SessionRepo> SessionDeleteByUserCommandService<Txn>
    for SessionDeleteByUserCommandServiceImpl<Auth, SessionRepo>
where
    Txn: Send + Sync + 'static,
    Auth: AuthService<Txn>,
    SessionRepo: SessionRepository<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, user_id: UserId) -> anyhow::Result<()> {
        self.auth.invalidate_access_tokens(txn, user_id).await?;
        self.session_repo.delete_by_user(txn, user_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use academy_auth_contracts::MockAuthService;
    use academy_demo::user::FOO;
    use academy_persistence_contracts::session::MockSessionRepository;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let auth = MockAuthService::new().with_invalidate_access_tokens(FOO.user.id);

        let session_repo = MockSessionRepository::new().with_delete_by_user(FOO.user.id);

        let sut = SessionDeleteByUserCommandServiceImpl { auth, session_repo };

        // Arrange
        let result = sut.invoke(&mut (), FOO.user.id).await;

        // Assert
        result.unwrap();
    }
}
