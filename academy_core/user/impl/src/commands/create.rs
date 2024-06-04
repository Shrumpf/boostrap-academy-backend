use academy_core_user_contracts::commands::create::{
    UserCreateCommand, UserCreateCommandError, UserCreateCommandService,
};
use academy_di::Build;
use academy_models::user::{User, UserComposite, UserDetails, UserProfile};
use academy_persistence_contracts::user::{UserRepoError, UserRepository};
use academy_shared_contracts::{id::IdService, password::PasswordService, time::TimeService};

#[derive(Debug, Clone, Copy, Build)]
pub struct UserCreateCommandServiceImpl<Id, Time, Password, UserRepo> {
    id: Id,
    time: Time,
    password: Password,
    user_repo: UserRepo,
}

impl<Txn, Id, Time, Password, UserRepo> UserCreateCommandService<Txn>
    for UserCreateCommandServiceImpl<Id, Time, Password, UserRepo>
where
    Txn: Send + Sync + 'static,
    Id: IdService,
    Time: TimeService,
    Password: PasswordService,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        UserCreateCommand {
            name,
            display_name,
            email,
            password,
            admin,
            enabled,
            email_verified,
        }: UserCreateCommand,
    ) -> Result<UserComposite, UserCreateCommandError> {
        let password_hash = self.password.hash(password.into_inner()).await?;

        let user = User {
            id: self.id.generate(),
            name,
            email: Some(email),
            email_verified,
            created_at: self.time.now(),
            last_login: None,
            last_name_change: None,
            enabled,
            admin,
            newsletter: false,
        };

        let profile = UserProfile {
            display_name,
            bio: Default::default(),
            tags: Default::default(),
        };

        let details = UserDetails { mfa_enabled: false };

        self.user_repo
            .create(txn, &user, &profile)
            .await
            .map_err(|err| match err {
                UserRepoError::NameConflict => UserCreateCommandError::NameConflict,
                UserRepoError::EmailConflict => UserCreateCommandError::EmailConflict,
                UserRepoError::Other(err) => UserCreateCommandError::Other(err),
            })?;

        self.user_repo
            .save_password_hash(txn, user.id, password_hash)
            .await?;

        let user_composite = UserComposite {
            user,
            profile,
            details,
        };

        Ok(user_composite)
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::user::FOO;
    use academy_models::user::UserPassword;
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_shared_contracts::{
        id::MockIdService, password::MockPasswordService, time::MockTimeService,
    };
    use academy_utils::assert_matches;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let user_password = UserPassword::try_new("secure password").unwrap();
        let user_password_hash = "password_hash".to_owned();

        let expected = get_expected();

        let id = MockIdService::new().with_generate(FOO.user.id);
        let time = MockTimeService::new().with_now(FOO.user.created_at);
        let password = MockPasswordService::new().with_hash(
            user_password.clone().into_inner(),
            user_password_hash.clone(),
        );
        let user_repo = MockUserRepository::new()
            .with_create(expected.user.clone(), expected.profile.clone(), Ok(()))
            .with_save_password_hash(FOO.user.id, user_password_hash);

        let sut = UserCreateCommandServiceImpl {
            id,
            time,
            password,
            user_repo,
        };

        let command = UserCreateCommand {
            name: FOO.user.name.clone(),
            display_name: FOO.profile.display_name.clone(),
            email: FOO.user.email.clone().unwrap(),
            password: user_password,
            admin: false,
            enabled: true,
            email_verified: false,
        };

        // Act
        let result = sut.invoke(&mut (), command).await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn name_conflict() {
        // Arrange
        let user_password = UserPassword::try_new("secure password").unwrap();
        let user_password_hash = "password_hash".to_owned();

        let expected = get_expected();

        let id = MockIdService::new().with_generate(FOO.user.id);
        let time = MockTimeService::new().with_now(FOO.user.created_at);
        let password = MockPasswordService::new().with_hash(
            user_password.clone().into_inner(),
            user_password_hash.clone(),
        );
        let user_repo = MockUserRepository::new().with_create(
            expected.user.clone(),
            expected.profile.clone(),
            Err(UserRepoError::NameConflict),
        );

        let sut = UserCreateCommandServiceImpl {
            id,
            time,
            password,
            user_repo,
        };

        let command = UserCreateCommand {
            name: FOO.user.name.clone(),
            display_name: FOO.profile.display_name.clone(),
            email: FOO.user.email.clone().unwrap(),
            password: user_password,
            admin: false,
            enabled: true,
            email_verified: false,
        };

        // Act
        let result = sut.invoke(&mut (), command).await;

        // Assert
        assert_matches!(result, Err(UserCreateCommandError::NameConflict));
    }

    #[tokio::test]
    async fn email_conflict() {
        // Arrange
        let user_password = UserPassword::try_new("secure password").unwrap();
        let user_password_hash = "password_hash".to_owned();

        let expected = get_expected();

        let id = MockIdService::new().with_generate(FOO.user.id);
        let time = MockTimeService::new().with_now(FOO.user.created_at);
        let password = MockPasswordService::new().with_hash(
            user_password.clone().into_inner(),
            user_password_hash.clone(),
        );
        let user_repo = MockUserRepository::new().with_create(
            expected.user.clone(),
            expected.profile.clone(),
            Err(UserRepoError::EmailConflict),
        );

        let sut = UserCreateCommandServiceImpl {
            id,
            time,
            password,
            user_repo,
        };

        let command = UserCreateCommand {
            name: FOO.user.name.clone(),
            display_name: FOO.profile.display_name.clone(),
            email: FOO.user.email.clone().unwrap(),
            password: user_password,
            admin: false,
            enabled: true,
            email_verified: false,
        };

        // Act
        let result = sut.invoke(&mut (), command).await;

        // Assert
        assert_matches!(result, Err(UserCreateCommandError::EmailConflict));
    }

    fn get_expected() -> UserComposite {
        UserComposite {
            user: User {
                id: FOO.user.id,
                name: FOO.user.name.clone(),
                email: FOO.user.email.clone(),
                email_verified: false,
                created_at: FOO.user.created_at,
                last_login: None,
                last_name_change: None,
                enabled: true,
                admin: false,
                newsletter: false,
            },
            profile: UserProfile {
                display_name: FOO.profile.display_name.clone(),
                bio: Default::default(),
                tags: Default::default(),
            },
            details: UserDetails { mfa_enabled: false },
        }
    }
}
