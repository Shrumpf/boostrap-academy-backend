use academy_auth_contracts::access_token::AuthAccessTokenService;
use academy_core_session_contracts::commands::delete::SessionDeleteCommandService;
use academy_di::Build;
use academy_models::session::SessionId;
use academy_persistence_contracts::session::SessionRepository;

#[derive(Debug, Clone, Default, Build)]
pub struct SessionDeleteCommandServiceImpl<AuthAccessToken, SessionRepo> {
    auth_access_token: AuthAccessToken,
    session_repo: SessionRepo,
}

impl<Txn, AuthAccessToken, SessionRepo> SessionDeleteCommandService<Txn>
    for SessionDeleteCommandServiceImpl<AuthAccessToken, SessionRepo>
where
    Txn: Send + Sync + 'static,
    AuthAccessToken: AuthAccessTokenService,
    SessionRepo: SessionRepository<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, session_id: SessionId) -> anyhow::Result<bool> {
        if let Some(refresh_token_hash) = self
            .session_repo
            .get_refresh_token_hash(txn, session_id)
            .await?
        {
            self.auth_access_token
                .invalidate(refresh_token_hash)
                .await?;
        }

        self.session_repo.delete(txn, session_id).await
    }
}

#[cfg(test)]
mod tests {
    use academy_auth_contracts::access_token::MockAuthAccessTokenService;
    use academy_demo::{session::FOO_1, SHA256HASH1};
    use academy_persistence_contracts::session::MockSessionRepository;

    use super::*;

    type Sut =
        SessionDeleteCommandServiceImpl<MockAuthAccessTokenService, MockSessionRepository<()>>;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let session_repo = MockSessionRepository::new()
            .with_get_refresh_token_hash(FOO_1.id, Some((*SHA256HASH1).into()))
            .with_delete(FOO_1.id, true);

        let auth_access_token =
            MockAuthAccessTokenService::new().with_invalidate((*SHA256HASH1).into());

        let sut = SessionDeleteCommandServiceImpl {
            auth_access_token,
            session_repo,
        };

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
