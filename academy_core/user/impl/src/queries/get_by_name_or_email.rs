use academy_core_user_contracts::queries::get_by_name_or_email::UserGetByNameOrEmailQueryService;
use academy_di::Build;
use academy_models::user::{UserComposite, UserNameOrEmailAddress};
use academy_persistence_contracts::user::UserRepository;

#[derive(Debug, Clone, Build)]
pub struct UserGetByNameOrEmailQueryServiceImpl<UserRepo> {
    user_repo: UserRepo,
}

impl<Txn, UserRepo> UserGetByNameOrEmailQueryService<Txn>
    for UserGetByNameOrEmailQueryServiceImpl<UserRepo>
where
    Txn: Send + Sync + 'static,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        name_or_email: &UserNameOrEmailAddress,
    ) -> anyhow::Result<Option<UserComposite>> {
        match name_or_email {
            UserNameOrEmailAddress::Name(name) => {
                self.user_repo.get_composite_by_name(txn, name).await
            }
            UserNameOrEmailAddress::Email(email) => {
                self.user_repo.get_composite_by_email(txn, email).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::user::FOO;
    use academy_persistence_contracts::user::MockUserRepository;

    use super::*;

    #[tokio::test]
    async fn by_name_ok() {
        // Arrange
        let user_repo = MockUserRepository::new()
            .with_get_composite_by_name(FOO.user.name.clone(), Some(FOO.clone()));
        let sut = UserGetByNameOrEmailQueryServiceImpl { user_repo };

        // Act
        let result = sut
            .invoke(
                &mut (),
                &UserNameOrEmailAddress::Name(FOO.user.name.clone()),
            )
            .await;

        // Assert
        assert_eq!(result.unwrap().unwrap(), *FOO);
    }

    #[tokio::test]
    async fn by_name_not_found() {
        // Arrange
        let user_repo =
            MockUserRepository::new().with_get_composite_by_name(FOO.user.name.clone(), None);
        let sut = UserGetByNameOrEmailQueryServiceImpl { user_repo };

        // Act
        let result = sut
            .invoke(
                &mut (),
                &UserNameOrEmailAddress::Name(FOO.user.name.clone()),
            )
            .await;

        // Assert
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn by_email_ok() {
        // Arrange
        let user_repo = MockUserRepository::new()
            .with_get_composite_by_email(FOO.user.email.clone().unwrap(), Some(FOO.clone()));
        let sut = UserGetByNameOrEmailQueryServiceImpl { user_repo };

        // Act
        let result = sut
            .invoke(
                &mut (),
                &UserNameOrEmailAddress::Email(FOO.user.email.clone().unwrap()),
            )
            .await;

        // Assert
        assert_eq!(result.unwrap().unwrap(), *FOO);
    }

    #[tokio::test]
    async fn by_email_not_found() {
        // Arrange
        let user_repo = MockUserRepository::new()
            .with_get_composite_by_email(FOO.user.email.clone().unwrap(), None);
        let sut = UserGetByNameOrEmailQueryServiceImpl { user_repo };

        // Act
        let result = sut
            .invoke(
                &mut (),
                &UserNameOrEmailAddress::Email(FOO.user.email.clone().unwrap()),
            )
            .await;

        // Assert
        assert_eq!(result.unwrap(), None);
    }
}
