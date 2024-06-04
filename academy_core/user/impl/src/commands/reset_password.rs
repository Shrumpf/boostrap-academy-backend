use academy_cache_contracts::CacheService;
use academy_core_user_contracts::commands::reset_password::{
    UserResetPasswordCommandError, UserResetPasswordCommandService,
};
use academy_di::Build;
use academy_models::{
    user::{UserId, UserPassword},
    VerificationCode,
};
use academy_persistence_contracts::user::UserRepository;
use academy_shared_contracts::password::PasswordService;

use crate::reset_password_cache_key;

#[derive(Debug, Clone, Build, Default)]
pub struct UserResetPasswordCommandServiceImpl<Cache, Password, UserRepo> {
    cache: Cache,
    password: Password,
    user_repo: UserRepo,
}

impl<Txn, Cache, Password, UserRepo> UserResetPasswordCommandService<Txn>
    for UserResetPasswordCommandServiceImpl<Cache, Password, UserRepo>
where
    Txn: Send + Sync + 'static,
    Cache: CacheService,
    Password: PasswordService,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        code: VerificationCode,
        new_password: UserPassword,
    ) -> Result<(), UserResetPasswordCommandError> {
        let cache_key = reset_password_cache_key(user_id);
        let expected_code: VerificationCode = self
            .cache
            .get(&cache_key)
            .await?
            .ok_or(UserResetPasswordCommandError::InvalidCode)?;
        if expected_code != code {
            return Err(UserResetPasswordCommandError::InvalidCode);
        }

        let password_hash = self.password.hash(new_password.into_inner()).await?;

        self.user_repo
            .save_password_hash(txn, user_id, password_hash)
            .await?;

        self.cache.remove(&cache_key).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use academy_cache_contracts::MockCacheService;
    use academy_demo::{
        user::{FOO, FOO_PASSWORD},
        VERIFICATION_CODE_1, VERIFICATION_CODE_2,
    };
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_shared_contracts::password::MockPasswordService;
    use academy_utils::assert_matches;

    use super::*;

    type Sut = UserResetPasswordCommandServiceImpl<
        MockCacheService,
        MockPasswordService,
        MockUserRepository<()>,
    >;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let cache_key = format!("reset_password_code:{}", FOO.user.id.hyphenated());
        let cache = MockCacheService::new()
            .with_get(cache_key.clone(), Some(VERIFICATION_CODE_1.clone()))
            .with_remove(cache_key);

        let password = MockPasswordService::new()
            .with_hash(FOO_PASSWORD.clone().into_inner(), "new pw hash".into());

        let user_repo =
            MockUserRepository::new().with_save_password_hash(FOO.user.id, "new pw hash".into());

        let sut = UserResetPasswordCommandServiceImpl {
            cache,
            password,
            user_repo,
        };

        // Act
        let result = sut
            .invoke(
                &mut (),
                FOO.user.id,
                VERIFICATION_CODE_1.clone(),
                FOO_PASSWORD.clone(),
            )
            .await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn no_code() {
        // Arrange
        let cache = MockCacheService::new().with_get(
            format!("reset_password_code:{}", FOO.user.id.hyphenated()),
            None::<VerificationCode>,
        );

        let sut = UserResetPasswordCommandServiceImpl {
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .invoke(
                &mut (),
                FOO.user.id,
                VERIFICATION_CODE_1.clone(),
                FOO_PASSWORD.clone(),
            )
            .await;

        // Assert
        assert_matches!(result, Err(UserResetPasswordCommandError::InvalidCode));
    }

    #[tokio::test]
    async fn invalid_code() {
        // Arrange
        let cache = MockCacheService::new().with_get(
            format!("reset_password_code:{}", FOO.user.id.hyphenated()),
            Some(VERIFICATION_CODE_2.clone()),
        );

        let sut = UserResetPasswordCommandServiceImpl {
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .invoke(
                &mut (),
                FOO.user.id,
                VERIFICATION_CODE_1.clone(),
                FOO_PASSWORD.clone(),
            )
            .await;

        // Assert
        assert_matches!(result, Err(UserResetPasswordCommandError::InvalidCode));
    }
}
