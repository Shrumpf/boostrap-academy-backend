use academy_auth_contracts::AuthService;
use academy_core_user_contracts::commands::update_email::{
    UserUpdateEmailCommandError, UserUpdateEmailCommandService,
};
use academy_di::Build;
use academy_models::{
    email_address::EmailAddress,
    user::{UserId, UserPatchRef},
};
use academy_persistence_contracts::user::{UserRepoError, UserRepository};

#[derive(Debug, Clone, Build)]
pub struct UserUpdateEmailCommandServiceImpl<Auth, UserRepo> {
    auth: Auth,
    user_repo: UserRepo,
}

impl<Txn, Auth, UserRepo> UserUpdateEmailCommandService<Txn>
    for UserUpdateEmailCommandServiceImpl<Auth, UserRepo>
where
    Txn: Send + Sync + 'static,
    Auth: AuthService<Txn>,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        email: &Option<EmailAddress>,
        email_verified: bool,
    ) -> Result<bool, UserUpdateEmailCommandError> {
        let result = self
            .user_repo
            .update(
                txn,
                user_id,
                UserPatchRef::new()
                    .update_email(email)
                    .update_email_verified(&email_verified),
            )
            .await
            .map_err(|err| match err {
                UserRepoError::EmailConflict => UserUpdateEmailCommandError::Conflict,
                err => UserUpdateEmailCommandError::Other(err.into()),
            })?;

        if result {
            self.auth.invalidate_access_tokens(txn, user_id).await?;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use academy_auth_contracts::MockAuthService;
    use academy_demo::user::{ADMIN, FOO};
    use academy_models::user::UserPatch;
    use academy_persistence_contracts::user::{MockUserRepository, UserRepoError};
    use academy_utils::assert_matches;

    use super::*;

    #[tokio::test]
    async fn ok() {
        for verified in [true, false] {
            // Arrange
            let auth = MockAuthService::new().with_invalidate_access_tokens(FOO.user.id);

            let user_repo = MockUserRepository::new().with_update(
                FOO.user.id,
                UserPatch::new()
                    .update_email(Some(ADMIN.user.email.clone().unwrap()))
                    .update_email_verified(verified),
                Ok(true),
            );

            let sut = UserUpdateEmailCommandServiceImpl { auth, user_repo };

            // Act
            let result = sut
                .invoke(
                    &mut (),
                    FOO.user.id,
                    &ADMIN.user.email.clone().unwrap().into(),
                    verified,
                )
                .await;

            // Assert
            assert!(result.unwrap());
        }
    }

    #[tokio::test]
    async fn conflict() {
        // Arrange
        let auth = MockAuthService::new();

        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new()
                .update_email(Some(ADMIN.user.email.clone().unwrap()))
                .update_email_verified(false),
            Err(UserRepoError::EmailConflict),
        );

        let sut = UserUpdateEmailCommandServiceImpl { auth, user_repo };

        // Act
        let result = sut
            .invoke(
                &mut (),
                FOO.user.id,
                &ADMIN.user.email.clone().unwrap().into(),
                false,
            )
            .await;

        // Assert
        assert_matches!(result, Err(UserUpdateEmailCommandError::Conflict));
    }
}
