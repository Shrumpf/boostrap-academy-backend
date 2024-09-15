use academy_auth_contracts::{access_token::AuthAccessTokenService, AuthService};
use academy_core_session_contracts::commands::refresh::{
    SessionRefreshCommandError, SessionRefreshCommandService,
};
use academy_di::Build;
use academy_models::{
    auth::Login,
    session::{SessionId, SessionPatch},
};
use academy_persistence_contracts::{session::SessionRepository, user::UserRepository};
use academy_shared_contracts::time::TimeService;
use academy_utils::patch::Patch;

#[derive(Debug, Clone, Default, Build)]
pub struct SessionRefreshCommandServiceImpl<Time, Auth, AuthAccessToken, UserRepo, SessionRepo> {
    time: Time,
    auth: Auth,
    auth_access_token: AuthAccessToken,
    user_repo: UserRepo,
    session_repo: SessionRepo,
}

impl<Txn, Time, Auth, AuthAccessToken, UserRepo, SessionRepo> SessionRefreshCommandService<Txn>
    for SessionRefreshCommandServiceImpl<Time, Auth, AuthAccessToken, UserRepo, SessionRepo>
where
    Txn: Send + Sync + 'static,
    Time: TimeService,
    Auth: AuthService<Txn>,
    AuthAccessToken: AuthAccessTokenService,
    UserRepo: UserRepository<Txn>,
    SessionRepo: SessionRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        session_id: SessionId,
    ) -> Result<Login, SessionRefreshCommandError> {
        let refresh_token_hash = self
            .session_repo
            .get_refresh_token_hash(txn, session_id)
            .await?
            .ok_or(SessionRefreshCommandError::NotFound)?;

        let session = self
            .session_repo
            .get(txn, session_id)
            .await?
            .ok_or(SessionRefreshCommandError::NotFound)?;

        let user_composite = self
            .user_repo
            .get_composite(txn, session.user_id)
            .await?
            .ok_or(SessionRefreshCommandError::NotFound)?;

        self.auth_access_token
            .invalidate(refresh_token_hash)
            .await?;

        let tokens = self.auth.issue_tokens(&user_composite.user, session_id)?;

        let now = self.time.now();

        let patch = SessionPatch::new().update_updated_at(now);
        self.session_repo
            .update(txn, session.id, patch.as_ref())
            .await?;
        let session = session.update(patch);

        self.session_repo
            .save_refresh_token_hash(txn, session.id, tokens.refresh_token_hash)
            .await?;

        Ok(Login {
            user_composite,
            session,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use academy_auth_contracts::{
        access_token::MockAuthAccessTokenService, MockAuthService, Tokens,
    };
    use academy_demo::{session::FOO_1, user::FOO, SHA256HASH1, SHA256HASH2};
    use academy_models::session::Session;
    use academy_persistence_contracts::{session::MockSessionRepository, user::MockUserRepository};
    use academy_shared_contracts::time::MockTimeService;
    use academy_utils::assert_matches;

    use super::*;

    type Sut = SessionRefreshCommandServiceImpl<
        MockTimeService,
        MockAuthService<()>,
        MockAuthAccessTokenService,
        MockUserRepository<()>,
        MockSessionRepository<()>,
    >;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let tokens = Tokens {
            access_token: "the new access token".into(),
            refresh_token: "the new refresh token".into(),
            refresh_token_hash: (*SHA256HASH2).into(),
        };

        let expected = Login {
            user_composite: FOO.clone(),
            session: Session {
                updated_at: FOO_1.updated_at + Duration::from_secs(3600),
                ..FOO_1.clone()
            },
            access_token: tokens.access_token.clone(),
            refresh_token: tokens.refresh_token.clone(),
        };

        let auth = MockAuthService::new().with_issue_tokens(FOO.user.clone(), FOO_1.id, tokens);

        let auth_access_token =
            MockAuthAccessTokenService::new().with_invalidate((*SHA256HASH1).into());

        let time = MockTimeService::new().with_now(expected.session.updated_at);

        let user_repo =
            MockUserRepository::new().with_get_composite(FOO_1.user_id, Some(FOO.clone()));
        let session_repo = MockSessionRepository::new()
            .with_get_refresh_token_hash(FOO_1.id, Some((*SHA256HASH1).into()))
            .with_get(FOO_1.id, Some(FOO_1.clone()))
            .with_update(
                FOO_1.id,
                SessionPatch::new().update_updated_at(expected.session.updated_at),
                true,
            )
            .with_save_refresh_token_hash(FOO_1.id, (*SHA256HASH2).into());

        let sut = SessionRefreshCommandServiceImpl {
            auth,
            auth_access_token,
            time,
            session_repo,
            user_repo,
        };

        // Act
        let result = sut.invoke(&mut (), FOO_1.id).await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn refresh_token_hash_not_found() {
        // Arrange
        let session_repo = MockSessionRepository::new().with_get_refresh_token_hash(FOO_1.id, None);

        let sut = SessionRefreshCommandServiceImpl {
            session_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.invoke(&mut (), FOO_1.id).await;

        // Assert
        assert_matches!(result, Err(SessionRefreshCommandError::NotFound));
    }

    #[tokio::test]
    async fn session_not_found() {
        // Arrange
        let session_repo = MockSessionRepository::new()
            .with_get_refresh_token_hash(FOO_1.id, Some((*SHA256HASH1).into()))
            .with_get(FOO_1.id, None);

        let sut = SessionRefreshCommandServiceImpl {
            session_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.invoke(&mut (), FOO_1.id).await;

        // Assert
        assert_matches!(result, Err(SessionRefreshCommandError::NotFound));
    }

    #[tokio::test]
    async fn user_not_found() {
        // Arrange
        let session_repo = MockSessionRepository::new()
            .with_get_refresh_token_hash(FOO_1.id, Some((*SHA256HASH1).into()))
            .with_get(FOO_1.id, Some(FOO_1.clone()));
        let user_repo = MockUserRepository::new().with_get_composite(FOO_1.user_id, None);

        let sut = SessionRefreshCommandServiceImpl {
            session_repo,
            user_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.invoke(&mut (), FOO_1.id).await;

        // Assert
        assert_matches!(result, Err(SessionRefreshCommandError::NotFound));
    }
}
