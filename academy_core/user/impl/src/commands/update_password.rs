use academy_core_user_contracts::commands::update_password::UserUpdatePasswordCommandService;
use academy_di::Build;
use academy_models::user::{UserId, UserPassword};
use academy_persistence_contracts::user::UserRepository;
use academy_shared_contracts::password::PasswordService;

#[derive(Debug, Clone, Build)]
pub struct UserUpdatePasswordCommandServiceImpl<Password, UserRepo> {
    password: Password,
    user_repo: UserRepo,
}

impl<Txn, Password, UserRepo> UserUpdatePasswordCommandService<Txn>
    for UserUpdatePasswordCommandServiceImpl<Password, UserRepo>
where
    Txn: Send + Sync + 'static,
    Password: PasswordService,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        password: UserPassword,
    ) -> anyhow::Result<()> {
        let hash = self.password.hash(password.into_inner()).await?;

        self.user_repo
            .save_password_hash(txn, user_id, hash)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::user::FOO;
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_shared_contracts::password::MockPasswordService;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let password =
            MockPasswordService::new().with_hash("new password".into(), "the hash".into());

        let user_repo =
            MockUserRepository::new().with_save_password_hash(FOO.user.id, "the hash".into());

        let sut = UserUpdatePasswordCommandServiceImpl {
            password,
            user_repo,
        };

        // Act
        let result = sut
            .invoke(&mut (), FOO.user.id, "new password".try_into().unwrap())
            .await;

        // Assert
        result.unwrap();
    }
}
