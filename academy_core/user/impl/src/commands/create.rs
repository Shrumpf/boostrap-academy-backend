use academy_core_oauth2_contracts::create_link::{
    OAuth2CreateLinkService, OAuth2CreateLinkServiceError,
};
use academy_core_user_contracts::commands::create::{
    UserCreateCommand, UserCreateCommandError, UserCreateCommandService,
};
use academy_di::Build;
use academy_models::user::{User, UserComposite, UserDetails, UserInvoiceInfo, UserProfile};
use academy_persistence_contracts::user::{UserRepoError, UserRepository};
use academy_shared_contracts::{id::IdService, password::PasswordService, time::TimeService};

#[derive(Debug, Clone, Copy, Build, Default)]
pub struct UserCreateCommandServiceImpl<Id, Time, Password, UserRepo, OAuth2CreateLink> {
    id: Id,
    time: Time,
    password: Password,
    user_repo: UserRepo,
    oauth2_create_link: OAuth2CreateLink,
}

impl<Txn, Id, Time, Password, UserRepo, OAuth2CreateLink> UserCreateCommandService<Txn>
    for UserCreateCommandServiceImpl<Id, Time, Password, UserRepo, OAuth2CreateLink>
where
    Txn: Send + Sync + 'static,
    Id: IdService,
    Time: TimeService,
    Password: PasswordService,
    UserRepo: UserRepository<Txn>,
    OAuth2CreateLink: OAuth2CreateLinkService<Txn>,
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
            oauth2_registration,
        }: UserCreateCommand,
    ) -> Result<UserComposite, UserCreateCommandError> {
        let password_hash = match password {
            Some(password) => Some(self.password.hash(password.into_inner()).await?),
            None => None,
        };

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

        let details = UserDetails {
            mfa_enabled: false,
            password_login: password_hash.is_some(),
            oauth2_login: oauth2_registration.is_some(),
        };

        let invoice_info = UserInvoiceInfo::default();

        self.user_repo
            .create(txn, &user, &profile, &invoice_info)
            .await
            .map_err(|err| match err {
                UserRepoError::NameConflict => UserCreateCommandError::NameConflict,
                UserRepoError::EmailConflict => UserCreateCommandError::EmailConflict,
                UserRepoError::Other(err) => UserCreateCommandError::Other(err),
            })?;

        if let Some(password_hash) = password_hash {
            self.user_repo
                .save_password_hash(txn, user.id, password_hash)
                .await?;
        }

        if let Some(oauth2_registration) = oauth2_registration {
            self.oauth2_create_link
                .invoke(
                    txn,
                    user.id,
                    oauth2_registration.provider_id,
                    oauth2_registration.remote_user,
                )
                .await
                .map_err(|err| match err {
                    OAuth2CreateLinkServiceError::RemoteAlreadyLinked => {
                        UserCreateCommandError::RemoteAlreadyLinked
                    }
                    OAuth2CreateLinkServiceError::Other(err) => err.into(),
                })?;
        }

        let user_composite = UserComposite {
            user,
            profile,
            details,
            invoice_info,
        };

        Ok(user_composite)
    }
}

#[cfg(test)]
mod tests {
    use academy_core_oauth2_contracts::create_link::MockOAuth2CreateLinkService;
    use academy_demo::{
        oauth2::{FOO_OAUTH2_LINK_1, TEST_OAUTH2_PROVIDER_ID},
        user::FOO,
    };
    use academy_models::{oauth2::OAuth2Registration, user::UserPassword};
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_shared_contracts::{
        id::MockIdService, password::MockPasswordService, time::MockTimeService,
    };
    use academy_utils::assert_matches;

    use super::*;

    type Sut = UserCreateCommandServiceImpl<
        MockIdService,
        MockTimeService,
        MockPasswordService,
        MockUserRepository<()>,
        MockOAuth2CreateLinkService<()>,
    >;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let user_password = UserPassword::try_new("secure password").unwrap();
        let user_password_hash = "password_hash".to_owned();

        let expected = get_expected(true, false);

        let id = MockIdService::new().with_generate(FOO.user.id);
        let time = MockTimeService::new().with_now(FOO.user.created_at);
        let password = MockPasswordService::new().with_hash(
            user_password.clone().into_inner(),
            user_password_hash.clone(),
        );
        let user_repo = MockUserRepository::new()
            .with_create(
                expected.user.clone(),
                expected.profile.clone(),
                Default::default(),
                Ok(()),
            )
            .with_save_password_hash(FOO.user.id, user_password_hash);

        let sut = UserCreateCommandServiceImpl {
            id,
            time,
            password,
            user_repo,
            ..Sut::default()
        };

        let command = UserCreateCommand {
            name: FOO.user.name.clone(),
            display_name: FOO.profile.display_name.clone(),
            email: FOO.user.email.clone().unwrap(),
            password: Some(user_password),
            admin: false,
            enabled: true,
            email_verified: false,
            oauth2_registration: None,
        };

        // Act
        let result = sut.invoke(&mut (), command).await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn ok_oauth2() {
        // Arrange
        let expected = get_expected(false, true);

        let id = MockIdService::new().with_generate(FOO.user.id);
        let time = MockTimeService::new().with_now(FOO.user.created_at);
        let user_repo = MockUserRepository::new().with_create(
            expected.user.clone(),
            expected.profile.clone(),
            Default::default(),
            Ok(()),
        );

        let oauth2_create_link = MockOAuth2CreateLinkService::new().with_invoke(
            FOO.user.id,
            TEST_OAUTH2_PROVIDER_ID.clone(),
            FOO_OAUTH2_LINK_1.remote_user.clone(),
            Ok(FOO_OAUTH2_LINK_1.clone()),
        );

        let sut = UserCreateCommandServiceImpl {
            id,
            time,
            user_repo,
            oauth2_create_link,
            ..Sut::default()
        };

        let command = UserCreateCommand {
            name: FOO.user.name.clone(),
            display_name: FOO.profile.display_name.clone(),
            email: FOO.user.email.clone().unwrap(),
            password: None,
            admin: false,
            enabled: true,
            email_verified: false,
            oauth2_registration: Some(OAuth2Registration {
                provider_id: TEST_OAUTH2_PROVIDER_ID.clone(),
                remote_user: FOO_OAUTH2_LINK_1.remote_user.clone(),
            }),
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

        let expected = get_expected(true, false);

        let id = MockIdService::new().with_generate(FOO.user.id);
        let time = MockTimeService::new().with_now(FOO.user.created_at);
        let password = MockPasswordService::new().with_hash(
            user_password.clone().into_inner(),
            user_password_hash.clone(),
        );
        let user_repo = MockUserRepository::new().with_create(
            expected.user.clone(),
            expected.profile.clone(),
            Default::default(),
            Err(UserRepoError::NameConflict),
        );

        let sut = UserCreateCommandServiceImpl {
            id,
            time,
            password,
            user_repo,
            ..Sut::default()
        };

        let command = UserCreateCommand {
            name: FOO.user.name.clone(),
            display_name: FOO.profile.display_name.clone(),
            email: FOO.user.email.clone().unwrap(),
            password: Some(user_password),
            admin: false,
            enabled: true,
            email_verified: false,
            oauth2_registration: None,
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

        let expected = get_expected(true, false);

        let id = MockIdService::new().with_generate(FOO.user.id);
        let time = MockTimeService::new().with_now(FOO.user.created_at);
        let password = MockPasswordService::new().with_hash(
            user_password.clone().into_inner(),
            user_password_hash.clone(),
        );
        let user_repo = MockUserRepository::new().with_create(
            expected.user.clone(),
            expected.profile.clone(),
            Default::default(),
            Err(UserRepoError::EmailConflict),
        );

        let sut = UserCreateCommandServiceImpl {
            id,
            time,
            password,
            user_repo,
            ..Sut::default()
        };

        let command = UserCreateCommand {
            name: FOO.user.name.clone(),
            display_name: FOO.profile.display_name.clone(),
            email: FOO.user.email.clone().unwrap(),
            password: Some(user_password),
            admin: false,
            enabled: true,
            email_verified: false,
            oauth2_registration: None,
        };

        // Act
        let result = sut.invoke(&mut (), command).await;

        // Assert
        assert_matches!(result, Err(UserCreateCommandError::EmailConflict));
    }

    #[tokio::test]
    async fn oauth2_remote_already_linked() {
        // Arrange
        let expected = get_expected(false, true);

        let id = MockIdService::new().with_generate(FOO.user.id);
        let time = MockTimeService::new().with_now(FOO.user.created_at);
        let user_repo = MockUserRepository::new().with_create(
            expected.user.clone(),
            expected.profile.clone(),
            Default::default(),
            Ok(()),
        );

        let oauth2_create_link = MockOAuth2CreateLinkService::new().with_invoke(
            FOO.user.id,
            TEST_OAUTH2_PROVIDER_ID.clone(),
            FOO_OAUTH2_LINK_1.remote_user.clone(),
            Err(OAuth2CreateLinkServiceError::RemoteAlreadyLinked),
        );

        let sut = UserCreateCommandServiceImpl {
            id,
            time,
            user_repo,
            oauth2_create_link,
            ..Sut::default()
        };

        let command = UserCreateCommand {
            name: FOO.user.name.clone(),
            display_name: FOO.profile.display_name.clone(),
            email: FOO.user.email.clone().unwrap(),
            password: None,
            admin: false,
            enabled: true,
            email_verified: false,
            oauth2_registration: Some(OAuth2Registration {
                provider_id: TEST_OAUTH2_PROVIDER_ID.clone(),
                remote_user: FOO_OAUTH2_LINK_1.remote_user.clone(),
            }),
        };

        // Act
        let result = sut.invoke(&mut (), command).await;

        // Assert
        assert_matches!(result, Err(UserCreateCommandError::RemoteAlreadyLinked));
    }

    fn get_expected(password_login: bool, oauth2_login: bool) -> UserComposite {
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
            details: UserDetails {
                mfa_enabled: false,
                password_login,
                oauth2_login,
            },
            invoice_info: Default::default(),
        }
    }
}
