use academy_cache_contracts::CacheService;
use academy_core_auth_contracts::AuthService;
use academy_core_user_contracts::commands::verify_email::{
    UserVerifyEmailCommandError, UserVerifyEmailCommandService,
};
use academy_di::Build;
use academy_models::{
    user::{UserComposite, UserPatchRef},
    VerificationCode,
};
use academy_persistence_contracts::user::UserRepository;

use crate::verification_cache_key;

#[derive(Debug, Clone, Build)]
pub struct UserVerifyEmailCommandServiceImpl<Auth, Cache, UserRepo> {
    auth: Auth,
    cache: Cache,
    user_repo: UserRepo,
}

impl<Txn, Auth, Cache, UserRepo> UserVerifyEmailCommandService<Txn>
    for UserVerifyEmailCommandServiceImpl<Auth, Cache, UserRepo>
where
    Txn: Send + Sync + 'static,
    Auth: AuthService<Txn>,
    Cache: CacheService,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        verification_code: &VerificationCode,
    ) -> Result<UserComposite, UserVerifyEmailCommandError> {
        let cache_key = verification_cache_key(verification_code);
        let email = self
            .cache
            .get(&cache_key)
            .await?
            .ok_or(UserVerifyEmailCommandError::InvalidCode)?;

        let mut user_composite = self
            .user_repo
            .get_composite_by_email(txn, &email)
            .await?
            .ok_or(UserVerifyEmailCommandError::InvalidCode)?;

        if user_composite.user.email_verified {
            self.cache.remove(&cache_key).await?;
            return Err(UserVerifyEmailCommandError::AlreadyVerified);
        }

        user_composite.user.email_verified = true;
        self.user_repo
            .update(
                txn,
                user_composite.user.id,
                UserPatchRef::new().update_email_verified(&true),
            )
            .await
            .map_err(|err| UserVerifyEmailCommandError::Other(err.into()))?;

        self.auth
            .invalidate_access_tokens(txn, user_composite.user.id)
            .await?;

        self.cache.remove(&cache_key).await?;

        Ok(user_composite)
    }
}

#[cfg(test)]
mod tests {
    use academy_cache_contracts::MockCacheService;
    use academy_core_auth_contracts::MockAuthService;
    use academy_demo::{user::FOO, VERIFICATION_CODE_1};
    use academy_models::{email_address::EmailAddress, user::UserPatch};
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_utils::{assert_matches, Apply};

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let auth = MockAuthService::new().with_invalidate_access_tokens(FOO.user.id);

        let cache_key = format!("verification:{}", **VERIFICATION_CODE_1);
        let cache = MockCacheService::new()
            .with_get(cache_key.clone(), Some(FOO.user.email.clone().unwrap()))
            .with_remove(cache_key);

        let user_repo = MockUserRepository::new()
            .with_get_composite_by_email(
                FOO.user.email.clone().unwrap(),
                Some(FOO.clone().with(|u| u.user.email_verified = false)),
            )
            .with_update(
                FOO.user.id,
                UserPatch::new().update_email_verified(true),
                Ok(true),
            );

        let sut = UserVerifyEmailCommandServiceImpl {
            auth,
            cache,
            user_repo,
        };

        // Act
        let result = sut.invoke(&mut (), &VERIFICATION_CODE_1).await;

        // Assert
        assert_eq!(result.unwrap(), *FOO);
    }

    #[tokio::test]
    async fn invalid_code() {
        // Arrange
        let auth = MockAuthService::new();

        let cache = MockCacheService::new().with_get(
            format!("verification:{}", **VERIFICATION_CODE_1),
            None::<EmailAddress>,
        );

        let user_repo = MockUserRepository::new();

        let sut = UserVerifyEmailCommandServiceImpl {
            auth,
            cache,
            user_repo,
        };

        // Act
        let result = sut.invoke(&mut (), &VERIFICATION_CODE_1).await;

        // Assert
        assert_matches!(result, Err(UserVerifyEmailCommandError::InvalidCode));
    }

    #[tokio::test]
    async fn user_not_found() {
        // Arrange
        let auth = MockAuthService::new();

        let cache = MockCacheService::new().with_get(
            format!("verification:{}", **VERIFICATION_CODE_1),
            Some(FOO.user.email.clone().unwrap()),
        );

        let user_repo = MockUserRepository::new()
            .with_get_composite_by_email(FOO.user.email.clone().unwrap(), None);

        let sut = UserVerifyEmailCommandServiceImpl {
            auth,
            cache,
            user_repo,
        };

        // Act
        let result = sut.invoke(&mut (), &VERIFICATION_CODE_1).await;

        // Assert
        assert_matches!(result, Err(UserVerifyEmailCommandError::InvalidCode));
    }

    #[tokio::test]
    async fn already_verified() {
        // Arrange
        let auth = MockAuthService::new();

        let cache_key = format!("verification:{}", **VERIFICATION_CODE_1);
        let cache = MockCacheService::new()
            .with_get(cache_key.clone(), Some(FOO.user.email.clone().unwrap()))
            .with_remove(cache_key);

        let user_repo = MockUserRepository::new()
            .with_get_composite_by_email(FOO.user.email.clone().unwrap(), Some(FOO.clone()));

        let sut = UserVerifyEmailCommandServiceImpl {
            auth,
            cache,
            user_repo,
        };

        // Act
        let result = sut.invoke(&mut (), &VERIFICATION_CODE_1).await;

        // Assert
        assert_matches!(result, Err(UserVerifyEmailCommandError::AlreadyVerified));
    }
}
