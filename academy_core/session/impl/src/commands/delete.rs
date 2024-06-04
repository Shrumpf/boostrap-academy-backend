use academy_core_auth_contracts::AuthService;
use academy_core_session_contracts::commands::delete::SessionDeleteCommandService;
use academy_di::Build;
use academy_models::session::SessionId;
use academy_persistence_contracts::session::SessionRepository;

#[derive(Debug, Clone, Default, Build)]
pub struct SessionDeleteCommandServiceImpl<Auth, SessionRepo> {
    auth: Auth,
    session_repo: SessionRepo,
}

impl<Txn, Auth, SessionRepo> SessionDeleteCommandService<Txn>
    for SessionDeleteCommandServiceImpl<Auth, SessionRepo>
where
    Txn: Send + Sync + 'static,
    Auth: AuthService<Txn>,
    SessionRepo: SessionRepository<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, session_id: SessionId) -> anyhow::Result<bool> {
        if let Some(refresh_token_hash) = self
            .session_repo
            .get_refresh_token_hash(txn, session_id)
            .await?
        {
            self.auth
                .invalidate_access_token(refresh_token_hash)
                .await?;
        }

        self.session_repo.delete(txn, session_id).await
    }
}

#[cfg(test)]
mod tests {
    use academy_core_auth_contracts::MockAuthService;
    use academy_demo::{session::FOO_1, SHA256HASH1};
    use academy_persistence_contracts::session::MockSessionRepository;

    use super::*;

    type Sut = SessionDeleteCommandServiceImpl<MockAuthService<()>, MockSessionRepository<()>>;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let session_repo = MockSessionRepository::new()
            .with_get_refresh_token_hash(FOO_1.id, Some((*SHA256HASH1).into()))
            .with_delete(FOO_1.id, true);

        let auth = MockAuthService::new().with_invalidate_access_token((*SHA256HASH1).into());

        let sut = SessionDeleteCommandServiceImpl { auth, session_repo };

        // Act
        let result = sut.invoke(&mut (), FOO_1.id).await;

        // Assert
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn not_found() {
        // Arrange
        let session_repo = MockSessionRepository::new()
            .with_get_refresh_token_hash(FOO_1.id, None)
            .with_delete(FOO_1.id, false);

        let sut = SessionDeleteCommandServiceImpl {
            session_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.invoke(&mut (), FOO_1.id).await;

        // Assert
        assert!(!result.unwrap());
    }
}
