use std::time::Duration;

use academy_core_user_contracts::commands::update_name::{
    UserUpdateNameCommandError, UserUpdateNameCommandService, UserUpdateNameRateLimitPolicy,
};
use academy_di::Build;
use academy_models::user::{User, UserName, UserPatch};
use academy_persistence_contracts::user::{UserRepoError, UserRepository};
use academy_shared_contracts::time::TimeService;
use academy_utils::patch::{Patch, PatchValue};

#[derive(Debug, Clone, Build)]
pub struct UserUpdateNameCommandServiceImpl<Time, UserRepo> {
    time: Time,
    user_repo: UserRepo,
    config: UserUpdateNameCommandServiceConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct UserUpdateNameCommandServiceConfig {
    pub name_change_rate_limit: Duration,
}

impl<Txn, Time, UserRepo> UserUpdateNameCommandService<Txn>
    for UserUpdateNameCommandServiceImpl<Time, UserRepo>
where
    Txn: Send + Sync + 'static,
    Time: TimeService,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        user: User,
        name: UserName,
        rate_limit_policy: UserUpdateNameRateLimitPolicy,
    ) -> Result<User, UserUpdateNameCommandError> {
        let last_name_change = match rate_limit_policy {
            UserUpdateNameRateLimitPolicy::Enforce => {
                let now = self.time.now();
                if let Some(last_name_change) = user.last_name_change {
                    let rate_limit_until = last_name_change + self.config.name_change_rate_limit;
                    if now < rate_limit_until {
                        return Err(UserUpdateNameCommandError::RateLimit {
                            until: rate_limit_until,
                        });
                    }
                }
                PatchValue::Update(Some(now))
            }
            UserUpdateNameRateLimitPolicy::Bypass => PatchValue::Unchanged,
        };

        let patch = UserPatch {
            name: name.into(),
            last_name_change,
            ..Default::default()
        };

        self.user_repo
            .update(txn, user.id, patch.as_ref())
            .await
            .map(|_| user.update(patch))
            .map_err(|err| match err {
                UserRepoError::NameConflict => UserUpdateNameCommandError::Conflict,
                err => UserUpdateNameCommandError::Other(err.into()),
            })
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::user::{BAR, FOO};
    use academy_models::user::UserPatch;
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_shared_contracts::time::MockTimeService;
    use academy_utils::assert_matches;

    use super::*;

    #[tokio::test]
    async fn ok_rate_limit() {
        // Arrange
        let config = UserUpdateNameCommandServiceConfig::default();

        let now = FOO.user.last_name_change.unwrap()
            + config.name_change_rate_limit
            + Duration::from_secs(2);

        let expected = User {
            name: BAR.user.name.clone(),
            last_name_change: Some(now),
            ..FOO.user.clone()
        };

        let time = MockTimeService::new().with_now(now);

        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new()
                .update_name(BAR.user.name.clone())
                .update_last_name_change(Some(now)),
            Ok(true),
        );

        let sut = UserUpdateNameCommandServiceImpl {
            time,
            user_repo,
            config,
        };

        // Act
        let result = sut
            .invoke(
                &mut (),
                FOO.user.clone(),
                BAR.user.name.clone(),
                UserUpdateNameRateLimitPolicy::Enforce,
            )
            .await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn ok_bypass_rate_limit() {
        // Arrange
        let config = UserUpdateNameCommandServiceConfig::default();

        let expected = User {
            name: BAR.user.name.clone(),
            ..FOO.user.clone()
        };

        let time = MockTimeService::new();

        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new().update_name(BAR.user.name.clone()),
            Ok(true),
        );

        let sut = UserUpdateNameCommandServiceImpl {
            time,
            user_repo,
            config,
        };

        // Act
        let result = sut
            .invoke(
                &mut (),
                FOO.user.clone(),
                BAR.user.name.clone(),
                UserUpdateNameRateLimitPolicy::Bypass,
            )
            .await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn rate_limit() {
        // Arrange
        let config = UserUpdateNameCommandServiceConfig::default();

        let time = MockTimeService::new().with_now(
            FOO.user.last_name_change.unwrap() + config.name_change_rate_limit
                - Duration::from_secs(2),
        );

        let user_repo = MockUserRepository::new();

        let sut = UserUpdateNameCommandServiceImpl {
            time,
            user_repo,
            config,
        };

        let expected = FOO.user.last_name_change.unwrap() + config.name_change_rate_limit;

        // Act
        let result = sut
            .invoke(
                &mut (),
                FOO.user.clone(),
                BAR.user.name.clone(),
                UserUpdateNameRateLimitPolicy::Enforce,
            )
            .await;

        // Assert
        assert_matches!(result, Err(UserUpdateNameCommandError::RateLimit { until }) if *until == expected);
    }

    #[tokio::test]
    async fn conflict() {
        // Arrange
        let config = UserUpdateNameCommandServiceConfig::default();

        let time = MockTimeService::new();

        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new().update_name(BAR.user.name.clone()),
            Err(UserRepoError::NameConflict),
        );

        let sut = UserUpdateNameCommandServiceImpl {
            time,
            user_repo,
            config,
        };

        // Act
        let result = sut
            .invoke(
                &mut (),
                FOO.user.clone(),
                BAR.user.name.clone(),
                UserUpdateNameRateLimitPolicy::Bypass,
            )
            .await;

        // Assert
        assert_matches!(result, Err(UserUpdateNameCommandError::Conflict));
    }

    impl Default for UserUpdateNameCommandServiceConfig {
        fn default() -> Self {
            Self {
                name_change_rate_limit: Duration::from_secs(30 * 24 * 3600),
            }
        }
    }
}
