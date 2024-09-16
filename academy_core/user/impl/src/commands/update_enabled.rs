use academy_core_session_contracts::session::SessionService;
use academy_core_user_contracts::commands::update_enabled::UserUpdateEnabledCommandService;
use academy_di::Build;
use academy_models::user::{UserId, UserPatchRef};
use academy_persistence_contracts::user::UserRepository;

#[derive(Debug, Clone, Build, Default)]
pub struct UserUpdateEnabledCommandServiceImpl<UserRepo, Session> {
    user_repo: UserRepo,
    session: Session,
}

impl<Txn, UserRepo, Session> UserUpdateEnabledCommandService<Txn>
    for UserUpdateEnabledCommandServiceImpl<UserRepo, Session>
where
    Txn: Send + Sync + 'static,
    UserRepo: UserRepository<Txn>,
    Session: SessionService<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, user_id: UserId, enabled: bool) -> anyhow::Result<bool> {
        if !enabled {
            self.session.delete_by_user(txn, user_id).await?;
        }

        self.user_repo
            .update(txn, user_id, UserPatchRef::new().update_enabled(&enabled))
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use academy_core_session_contracts::session::MockSessionService;
    use academy_demo::user::{BAR, FOO};
    use academy_models::user::UserPatch;
    use academy_persistence_contracts::user::MockUserRepository;

    use super::*;

    type Sut = UserUpdateEnabledCommandServiceImpl<MockUserRepository<()>, MockSessionService<()>>;

    #[tokio::test]
    async fn enable() {
        // Arrange
        let user_repo = MockUserRepository::new().with_update(
            BAR.user.id,
            UserPatch::new().update_enabled(true),
            Ok(true),
        );

        let sut = UserUpdateEnabledCommandServiceImpl {
            user_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.invoke(&mut (), BAR.user.id, true).await;

        // Act
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn disable() {
        // Arrange
        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new().update_enabled(false),
            Ok(true),
        );

        let session = MockSessionService::new().with_delete_by_user(FOO.user.id);

        let sut = UserUpdateEnabledCommandServiceImpl { user_repo, session };

        // Act
        let result = sut.invoke(&mut (), FOO.user.id, false).await;

        // Act
        assert!(result.unwrap());
    }
}
