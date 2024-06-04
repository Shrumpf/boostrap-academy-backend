use academy_cache_contracts::CacheService;
use academy_core_user_contracts::commands::verify_newsletter_subscription::{
    UserVerifyNewsletterSubscriptionCommandError, UserVerifyNewsletterSubscriptionCommandService,
};
use academy_di::Build;
use academy_models::{
    user::{UserId, UserPatchRef},
    VerificationCode,
};
use academy_persistence_contracts::user::UserRepository;

use crate::subscribe_newsletter_cache_key;

#[derive(Debug, Clone, Build)]
pub struct UserVerifyNewsletterSubscriptionCommandServiceImpl<UserRepo, Cache> {
    user_repo: UserRepo,
    cache: Cache,
}

impl<Txn, UserRepo, Cache> UserVerifyNewsletterSubscriptionCommandService<Txn>
    for UserVerifyNewsletterSubscriptionCommandServiceImpl<UserRepo, Cache>
where
    Txn: Send + Sync + 'static,
    UserRepo: UserRepository<Txn>,
    Cache: CacheService,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        code: VerificationCode,
    ) -> Result<(), UserVerifyNewsletterSubscriptionCommandError> {
        let cache_key = subscribe_newsletter_cache_key(user_id);

        if self.cache.get::<String>(&cache_key).await? != Some(code.into_inner()) {
            return Err(UserVerifyNewsletterSubscriptionCommandError::InvalidCode);
        }

        self.user_repo
            .update(txn, user_id, UserPatchRef::new().update_newsletter(&true))
            .await
            .map_err(|err| UserVerifyNewsletterSubscriptionCommandError::Other(err.into()))?;

        self.cache.remove(&cache_key).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use academy_cache_contracts::MockCacheService;
    use academy_demo::{user::FOO, VERIFICATION_CODE_1, VERIFICATION_CODE_2};
    use academy_models::user::UserPatch;
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_utils::assert_matches;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let user_repo = MockUserRepository::new().with_update(
            FOO.user.id,
            UserPatch::new().update_newsletter(true),
            Ok(true),
        );

        let cache_key = format!("subscribe_newsletter_code:{}", FOO.user.id.hyphenated());
        let cache = MockCacheService::new()
            .with_get(
                cache_key.clone(),
                VERIFICATION_CODE_1.clone().into_inner().into(),
            )
            .with_remove(cache_key);

        let sut = UserVerifyNewsletterSubscriptionCommandServiceImpl { user_repo, cache };

        // Act
        let result = sut
            .invoke(&mut (), FOO.user.id, VERIFICATION_CODE_1.clone())
            .await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn no_code_requested() {
        // Arrange
        let user_repo = MockUserRepository::new();

        let cache = MockCacheService::new().with_get(
            format!("subscribe_newsletter_code:{}", FOO.user.id.hyphenated()),
            None::<String>,
        );

        let sut = UserVerifyNewsletterSubscriptionCommandServiceImpl { user_repo, cache };

        // Act
        let result = sut
            .invoke(&mut (), FOO.user.id, VERIFICATION_CODE_1.clone())
            .await;

        // Assert
        assert_matches!(
            result,
            Err(UserVerifyNewsletterSubscriptionCommandError::InvalidCode)
        );
    }

    #[tokio::test]
    async fn invalid_code() {
        // Arrange
        let user_repo = MockUserRepository::new();

        let cache = MockCacheService::new().with_get(
            format!("subscribe_newsletter_code:{}", FOO.user.id.hyphenated()),
            VERIFICATION_CODE_2.clone().into_inner().into(),
        );

        let sut = UserVerifyNewsletterSubscriptionCommandServiceImpl { user_repo, cache };

        // Act
        let result = sut
            .invoke(&mut (), FOO.user.id, VERIFICATION_CODE_1.clone())
            .await;

        // Assert
        assert_matches!(
            result,
            Err(UserVerifyNewsletterSubscriptionCommandError::InvalidCode)
        );
    }
}
