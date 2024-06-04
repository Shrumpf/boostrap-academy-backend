use academy_core_auth_contracts::AuthService;
use academy_core_session_contracts::commands::create::SessionCreateCommandService;
use academy_di::Build;
use academy_models::{
    auth::Login,
    session::{DeviceName, Session},
    user::{UserComposite, UserPatch},
};
use academy_persistence_contracts::{session::SessionRepository, user::UserRepository};
use academy_shared_contracts::{id::IdService, time::TimeService};
use academy_utils::patch::Patch;

#[derive(Debug, Clone, Build)]
pub struct SessionCreateCommandServiceImpl<Id, Time, Auth, SessionRepo, UserRepo> {
    id: Id,
    time: Time,
    auth: Auth,
    session_repo: SessionRepo,
    user_repo: UserRepo,
}

impl<Txn, Id, Time, Auth, SessionRepo, UserRepo> SessionCreateCommandService<Txn>
    for SessionCreateCommandServiceImpl<Id, Time, Auth, SessionRepo, UserRepo>
where
    Txn: Send + Sync + 'static,
    Id: IdService,
    Time: TimeService,
    Auth: AuthService<Txn>,
    SessionRepo: SessionRepository<Txn>,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        mut user_composite: UserComposite,
        device_name: Option<DeviceName>,
        update_last_login: bool,
    ) -> anyhow::Result<Login> {
        let id = self.id.generate();
        let now = self.time.now();

        let session = Session {
            id,
            user_id: user_composite.user.id,
            device_name,
            created_at: now,
            updated_at: now,
        };

        let tokens = self.auth.issue_tokens(&user_composite.user, session.id)?;

        self.session_repo.create(txn, &session).await?;
        self.session_repo
            .save_refresh_token_hash(txn, session.id, tokens.refresh_token_hash)
            .await?;

        if update_last_login {
            let patch = UserPatch::new().update_last_login(Some(now));
            self.user_repo
                .update(txn, user_composite.user.id, patch.as_ref())
                .await?;
            user_composite.user = user_composite.user.update(patch);
        }

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
    use academy_core_auth_contracts::{MockAuthService, Tokens};
    use academy_demo::{session::FOO_1, user::FOO, SHA256HASH1};
    use academy_models::user::{User, UserPatch};
    use academy_persistence_contracts::{session::MockSessionRepository, user::MockUserRepository};
    use academy_shared_contracts::{id::MockIdService, time::MockTimeService};

    use super::*;

    #[tokio::test]
    async fn ok_update_last_login() {
        // Arrange
        let tokens = Tokens {
            access_token: "the access token".into(),
            refresh_token: "the refresh token".into(),
            refresh_token_hash: (*SHA256HASH1).into(),
        };

        let expected = Login {
            user_composite: UserComposite {
                user: User {
                    last_login: Some(FOO_1.created_at),
                    ..FOO.user.clone()
                },
                ..FOO.clone()
            },
            session: Session {
                id: FOO_1.id,
                user_id: FOO.user.id,
                device_name: FOO_1.device_name.clone(),
                created_at: FOO_1.created_at,
                updated_at: FOO_1.created_at,
            },
            access_token: tokens.access_token.clone(),
            refresh_token: tokens.refresh_token.clone(),
        };

        let id = MockIdService::new().with_generate(FOO_1.id);
        let time = MockTimeService::new().with_now(FOO_1.created_at);
        let auth =
            MockAuthService::new().with_issue_tokens(FOO.user.clone(), FOO_1.id, tokens.clone());
        let session_repo = MockSessionRepository::new()
            .with_create(expected.session.clone())
            .with_save_refresh_token_hash(FOO_1.id, (*SHA256HASH1).into());

        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new().update_last_login(Some(FOO_1.created_at)),
            Ok(true),
        );

        let sut = SessionCreateCommandServiceImpl {
            id,
            time,
            auth,
            session_repo,
            user_repo,
        };

        // Act
        let result = sut
            .invoke(&mut (), FOO.clone(), FOO_1.device_name.clone(), true)
            .await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn ok_dont_update_last_login() {
        // Arrange
        let tokens = Tokens {
            access_token: "the access token".into(),
            refresh_token: "the refresh token".into(),
            refresh_token_hash: (*SHA256HASH1).into(),
        };

        let expected = Login {
            user_composite: FOO.clone(),
            session: Session {
                id: FOO_1.id,
                user_id: FOO.user.id,
                device_name: FOO_1.device_name.clone(),
                created_at: FOO_1.created_at,
                updated_at: FOO_1.created_at,
            },
            access_token: tokens.access_token.clone(),
            refresh_token: tokens.refresh_token.clone(),
        };

        let id = MockIdService::new().with_generate(FOO_1.id);
        let time = MockTimeService::new().with_now(FOO_1.created_at);
        let auth =
            MockAuthService::new().with_issue_tokens(FOO.user.clone(), FOO_1.id, tokens.clone());
        let session_repo = MockSessionRepository::new()
            .with_create(expected.session.clone())
            .with_save_refresh_token_hash(FOO_1.id, (*SHA256HASH1).into());

        let user_repo = MockUserRepository::new();

        let sut = SessionCreateCommandServiceImpl {
            id,
            time,
            auth,
            session_repo,
            user_repo,
        };

        // Act
        let result = sut
            .invoke(&mut (), FOO.clone(), FOO_1.device_name.clone(), false)
            .await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }
}
