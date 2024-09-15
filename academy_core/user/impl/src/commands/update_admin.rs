use academy_auth_contracts::AuthService;
use academy_core_user_contracts::commands::update_admin::UserUpdateAdminCommandService;
use academy_di::Build;
use academy_models::user::{UserId, UserPatchRef};
use academy_persistence_contracts::user::UserRepository;

#[derive(Debug, Clone, Build)]
pub struct UserUpdateAdminCommandServiceImpl<Auth, UserRepo> {
    auth: Auth,
    user_repo: UserRepo,
}

impl<Txn, Auth, UserRepo> UserUpdateAdminCommandService<Txn>
    for UserUpdateAdminCommandServiceImpl<Auth, UserRepo>
where
    Txn: Send + Sync + 'static,
    Auth: AuthService<Txn>,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(&self, txn: &mut Txn, user_id: UserId, admin: bool) -> anyhow::Result<bool> {
        self.auth.invalidate_access_tokens(txn, user_id).await?;
        self.user_repo
            .update(txn, user_id, UserPatchRef::new().update_admin(&admin))
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use academy_auth_contracts::MockAuthService;
    use academy_demo::user::FOO;
    use academy_models::user::UserPatch;
    use academy_persistence_contracts::user::MockUserRepository;

    use super::*;

    #[tokio::test]
    async fn promote() {
        // Arrange
        let auth = MockAuthService::new().with_invalidate_access_tokens(FOO.user.id);

        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new().update_admin(true),
            Ok(true),
        );

        let sut = UserUpdateAdminCommandServiceImpl { auth, user_repo };

        // Act
        let result = sut.invoke(&mut (), FOO.user.id, true).await;

        // Assert
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn demote() {
        // Arrange
        let auth = MockAuthService::new().with_invalidate_access_tokens(FOO.user.id);

        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new().update_admin(false),
            Ok(true),
        );

        let sut = UserUpdateAdminCommandServiceImpl { auth, user_repo };

        // Act
        let result = sut.invoke(&mut (), FOO.user.id, false).await;

        // Assert
        assert!(result.unwrap());
    }
}
