use academy_auth_contracts::AuthService;
use academy_cache_contracts::CacheService;
use academy_core_user_contracts::email_confirmation::{
    UserEmailConfirmationResetPasswordError, UserEmailConfirmationService,
    UserEmailConfirmationSubscribeToNewsletterError, UserEmailConfirmationVerifyEmailError,
};
use academy_di::Build;
use academy_email_contracts::template::TemplateEmailService;
use academy_models::{
    email_address::EmailAddressWithName,
    user::{UserComposite, UserId, UserPassword, UserPatchRef},
    VerificationCode,
};
use academy_persistence_contracts::user::UserRepository;
use academy_shared_contracts::{password::PasswordService, secret::SecretService};
use academy_templates_contracts::{
    ResetPasswordTemplate, SubscribeNewsletterTemplate, VerifyEmailTemplate,
};

use crate::UserFeatureConfig;

#[derive(Debug, Clone, Build)]
#[cfg_attr(test, derive(Default))]
pub struct UserEmailConfirmationServiceImpl<Auth, Secret, TemplateEmail, Cache, Password, UserRepo>
{
    auth: Auth,
    secret: Secret,
    template_email: TemplateEmail,
    cache: Cache,
    password: Password,
    user_repo: UserRepo,
    config: UserFeatureConfig,
}

impl<Txn, Auth, Secret, TemplateEmail, Cache, Password, UserRepo> UserEmailConfirmationService<Txn>
    for UserEmailConfirmationServiceImpl<Auth, Secret, TemplateEmail, Cache, Password, UserRepo>
where
    Txn: Send + Sync + 'static,
    Auth: AuthService<Txn>,
    Secret: SecretService,
    TemplateEmail: TemplateEmailService,
    Cache: CacheService,
    Password: PasswordService,
    UserRepo: UserRepository<Txn>,
{
    async fn request_verification(&self, email: EmailAddressWithName) -> anyhow::Result<()> {
        let code = self.secret.generate_verification_code();

        self.cache
            .set(
                &verification_cache_key(&code),
                &email.clone().into_email_address(),
                Some(self.config.verification_verification_code_ttl),
            )
            .await?;

        self.template_email
            .send_verification_email(
                email,
                &VerifyEmailTemplate {
                    code: code.into_inner(),
                    url: (*self.config.verification_redirect_url).clone(),
                },
            )
            .await?;

        Ok(())
    }

    async fn verify_email(
        &self,
        txn: &mut Txn,
        verification_code: &VerificationCode,
    ) -> Result<UserComposite, UserEmailConfirmationVerifyEmailError> {
        let cache_key = verification_cache_key(verification_code);
        let email = self
            .cache
            .get(&cache_key)
            .await?
            .ok_or(UserEmailConfirmationVerifyEmailError::InvalidCode)?;

        let mut user_composite = self
            .user_repo
            .get_composite_by_email(txn, &email)
            .await?
            .ok_or(UserEmailConfirmationVerifyEmailError::InvalidCode)?;

        if user_composite.user.email_verified {
            self.cache.remove(&cache_key).await?;
            return Err(UserEmailConfirmationVerifyEmailError::AlreadyVerified);
        }

        user_composite.user.email_verified = true;
        self.user_repo
            .update(
                txn,
                user_composite.user.id,
                UserPatchRef::new().update_email_verified(&true),
            )
            .await
            .map_err(|err| UserEmailConfirmationVerifyEmailError::Other(err.into()))?;

        self.auth
            .invalidate_access_tokens(txn, user_composite.user.id)
            .await?;

        self.cache.remove(&cache_key).await?;

        Ok(user_composite)
    }

    async fn request_password_reset(
        &self,
        user_id: UserId,
        email: EmailAddressWithName,
    ) -> anyhow::Result<()> {
        let code = self.secret.generate_verification_code();

        self.cache
            .set(
                &reset_password_cache_key(user_id),
                &code,
                Some(self.config.password_reset_verification_code_ttl),
            )
            .await?;

        self.template_email
            .send_reset_password_email(
                email,
                &ResetPasswordTemplate {
                    code: code.into_inner(),
                    url: (*self.config.password_reset_redirect_url).clone(),
                },
            )
            .await?;

        Ok(())
    }

    async fn reset_password(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        code: VerificationCode,
        new_password: UserPassword,
    ) -> Result<(), UserEmailConfirmationResetPasswordError> {
        let cache_key = reset_password_cache_key(user_id);
        let expected_code: VerificationCode = self
            .cache
            .get(&cache_key)
            .await?
            .ok_or(UserEmailConfirmationResetPasswordError::InvalidCode)?;
        if expected_code != code {
            return Err(UserEmailConfirmationResetPasswordError::InvalidCode);
        }

        let password_hash = self.password.hash(new_password.into_inner()).await?;

        self.user_repo
            .save_password_hash(txn, user_id, password_hash)
            .await?;

        self.cache.remove(&cache_key).await?;

        Ok(())
    }

    async fn request_newsletter_subscription(
        &self,
        user_id: UserId,
        email: EmailAddressWithName,
    ) -> anyhow::Result<()> {
        let code = self.secret.generate_verification_code();

        self.cache
            .set(
                &subscribe_newsletter_cache_key(user_id),
                &*code,
                Some(self.config.newsletter_subscription_verification_code_ttl),
            )
            .await?;

        self.template_email
            .send_subscribe_newsletter_email(
                email,
                &SubscribeNewsletterTemplate {
                    code: code.into_inner(),
                    url: self.config.newsletter_subscription_redirect_url.to_string(),
                },
            )
            .await?;

        Ok(())
    }

    async fn subscribe_to_newsletter(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        code: VerificationCode,
    ) -> Result<(), UserEmailConfirmationSubscribeToNewsletterError> {
        let cache_key = subscribe_newsletter_cache_key(user_id);

        if self.cache.get::<String>(&cache_key).await? != Some(code.into_inner()) {
            return Err(UserEmailConfirmationSubscribeToNewsletterError::InvalidCode);
        }

        self.user_repo
            .update(txn, user_id, UserPatchRef::new().update_newsletter(&true))
            .await
            .map_err(|err| UserEmailConfirmationSubscribeToNewsletterError::Other(err.into()))?;

        self.cache.remove(&cache_key).await?;

        Ok(())
    }
}

fn verification_cache_key(verification_code: &VerificationCode) -> String {
    format!("verification:{}", **verification_code)
}

fn subscribe_newsletter_cache_key(user_id: UserId) -> String {
    format!("subscribe_newsletter_code:{}", user_id.hyphenated())
}

fn reset_password_cache_key(user_id: UserId) -> String {
    format!("reset_password_code:{}", user_id.hyphenated())
}

#[cfg(test)]
mod tests {
    use academy_auth_contracts::MockAuthService;
    use academy_cache_contracts::MockCacheService;
    use academy_demo::{
        user::{FOO, FOO_PASSWORD},
        VERIFICATION_CODE_1, VERIFICATION_CODE_2,
    };
    use academy_email_contracts::template::MockTemplateEmailService;
    use academy_models::{email_address::EmailAddress, user::UserPatch};
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_shared_contracts::{password::MockPasswordService, secret::MockSecretService};
    use academy_utils::{assert_matches, Apply};

    use super::*;

    type Sut = UserEmailConfirmationServiceImpl<
        MockAuthService<()>,
        MockSecretService,
        MockTemplateEmailService,
        MockCacheService,
        MockPasswordService,
        MockUserRepository<()>,
    >;

    #[tokio::test]
    async fn request_verification() {
        // Arrange
        let config = UserFeatureConfig::default();

        let recipient = FOO
            .user
            .email
            .clone()
            .unwrap()
            .with_name(FOO.profile.display_name.clone().into_inner());

        let secret =
            MockSecretService::new().with_generate_verification_code(VERIFICATION_CODE_1.clone());

        let template_email = MockTemplateEmailService::new().with_send_verification_email(
            recipient.clone(),
            VerifyEmailTemplate {
                code: VERIFICATION_CODE_1.clone().into_inner(),
                url: (*config.verification_redirect_url).clone(),
            },
            true,
        );

        let cache = MockCacheService::new().with_set(
            format!("verification:{}", **VERIFICATION_CODE_1),
            FOO.user.email.clone().unwrap(),
            Some(config.verification_verification_code_ttl),
        );

        let sut = UserEmailConfirmationServiceImpl {
            secret,
            template_email,
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut.request_verification(recipient).await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn verify_email_ok() {
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

        let sut = UserEmailConfirmationServiceImpl {
            auth,
            cache,
            user_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.verify_email(&mut (), &VERIFICATION_CODE_1).await;

        // Assert
        assert_eq!(result.unwrap(), *FOO);
    }

    #[tokio::test]
    async fn verify_email_invalid_code() {
        // Arrange
        let auth = MockAuthService::new();

        let cache = MockCacheService::new().with_get(
            format!("verification:{}", **VERIFICATION_CODE_1),
            None::<EmailAddress>,
        );

        let user_repo = MockUserRepository::new();

        let sut = UserEmailConfirmationServiceImpl {
            auth,
            cache,
            user_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.verify_email(&mut (), &VERIFICATION_CODE_1).await;

        // Assert
        assert_matches!(
            result,
            Err(UserEmailConfirmationVerifyEmailError::InvalidCode)
        );
    }

    #[tokio::test]
    async fn verify_email_user_not_found() {
        // Arrange
        let auth = MockAuthService::new();

        let cache = MockCacheService::new().with_get(
            format!("verification:{}", **VERIFICATION_CODE_1),
            Some(FOO.user.email.clone().unwrap()),
        );

        let user_repo = MockUserRepository::new()
            .with_get_composite_by_email(FOO.user.email.clone().unwrap(), None);

        let sut = UserEmailConfirmationServiceImpl {
            auth,
            cache,
            user_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.verify_email(&mut (), &VERIFICATION_CODE_1).await;

        // Assert
        assert_matches!(
            result,
            Err(UserEmailConfirmationVerifyEmailError::InvalidCode)
        );
    }

    #[tokio::test]
    async fn verify_email_already_verified() {
        // Arrange
        let auth = MockAuthService::new();

        let cache_key = format!("verification:{}", **VERIFICATION_CODE_1);
        let cache = MockCacheService::new()
            .with_get(cache_key.clone(), Some(FOO.user.email.clone().unwrap()))
            .with_remove(cache_key);

        let user_repo = MockUserRepository::new()
            .with_get_composite_by_email(FOO.user.email.clone().unwrap(), Some(FOO.clone()));

        let sut = UserEmailConfirmationServiceImpl {
            auth,
            cache,
            user_repo,
            ..Sut::default()
        };

        // Act
        let result = sut.verify_email(&mut (), &VERIFICATION_CODE_1).await;

        // Assert
        assert_matches!(
            result,
            Err(UserEmailConfirmationVerifyEmailError::AlreadyVerified)
        );
    }

    #[tokio::test]
    async fn request_password_reset() {
        // Arrange
        let config = UserFeatureConfig::default();

        let secret =
            MockSecretService::new().with_generate_verification_code(VERIFICATION_CODE_1.clone());

        let expected_email = ResetPasswordTemplate {
            code: VERIFICATION_CODE_1.clone().into_inner(),
            url: (*config.password_reset_redirect_url).clone(),
        };

        let template_email = MockTemplateEmailService::new().with_send_reset_password_email(
            FOO.user
                .email
                .clone()
                .unwrap()
                .with_name(FOO.profile.display_name.clone().into_inner()),
            expected_email,
            true,
        );

        let cache = MockCacheService::new().with_set(
            format!("reset_password_code:{}", FOO.user.id.hyphenated()),
            VERIFICATION_CODE_1.clone(),
            Some(config.password_reset_verification_code_ttl),
        );

        let sut = UserEmailConfirmationServiceImpl {
            secret,
            template_email,
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .request_password_reset(
                FOO.user.id,
                FOO.user
                    .email
                    .clone()
                    .unwrap()
                    .with_name(FOO.profile.display_name.clone().into_inner()),
            )
            .await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn reset_password_ok() {
        // Arrange
        let cache_key = format!("reset_password_code:{}", FOO.user.id.hyphenated());
        let cache = MockCacheService::new()
            .with_get(cache_key.clone(), Some(VERIFICATION_CODE_1.clone()))
            .with_remove(cache_key);

        let password = MockPasswordService::new()
            .with_hash(FOO_PASSWORD.clone().into_inner(), "new pw hash".into());

        let user_repo =
            MockUserRepository::new().with_save_password_hash(FOO.user.id, "new pw hash".into());

        let sut = UserEmailConfirmationServiceImpl {
            cache,
            password,
            user_repo,
            ..Sut::default()
        };

        // Act
        let result = sut
            .reset_password(
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
    async fn reset_password_no_code() {
        // Arrange
        let cache = MockCacheService::new().with_get(
            format!("reset_password_code:{}", FOO.user.id.hyphenated()),
            None::<VerificationCode>,
        );

        let sut = UserEmailConfirmationServiceImpl {
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .reset_password(
                &mut (),
                FOO.user.id,
                VERIFICATION_CODE_1.clone(),
                FOO_PASSWORD.clone(),
            )
            .await;

        // Assert
        assert_matches!(
            result,
            Err(UserEmailConfirmationResetPasswordError::InvalidCode)
        );
    }

    #[tokio::test]
    async fn reset_password_invalid_code() {
        // Arrange
        let cache = MockCacheService::new().with_get(
            format!("reset_password_code:{}", FOO.user.id.hyphenated()),
            Some(VERIFICATION_CODE_2.clone()),
        );

        let sut = UserEmailConfirmationServiceImpl {
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .reset_password(
                &mut (),
                FOO.user.id,
                VERIFICATION_CODE_1.clone(),
                FOO_PASSWORD.clone(),
            )
            .await;

        // Assert
        assert_matches!(
            result,
            Err(UserEmailConfirmationResetPasswordError::InvalidCode)
        );
    }

    #[tokio::test]
    async fn request_newsletter_subscription() {
        // Arrange
        let config = UserFeatureConfig::default();

        let secret =
            MockSecretService::new().with_generate_verification_code(VERIFICATION_CODE_1.clone());

        let expected_email = SubscribeNewsletterTemplate {
            code: VERIFICATION_CODE_1.clone().into_inner(),
            url: (*config.newsletter_subscription_redirect_url).clone(),
        };

        let template_email = MockTemplateEmailService::new().with_send_subscribe_newsletter_email(
            FOO.user
                .email
                .clone()
                .unwrap()
                .with_name(FOO.profile.display_name.clone().into_inner()),
            expected_email,
            true,
        );

        let cache = MockCacheService::new().with_set(
            format!("subscribe_newsletter_code:{}", FOO.user.id.hyphenated()),
            VERIFICATION_CODE_1.clone().into_inner(),
            Some(config.newsletter_subscription_verification_code_ttl),
        );

        let sut = UserEmailConfirmationServiceImpl {
            secret,
            template_email,
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .request_newsletter_subscription(
                FOO.user.id,
                FOO.user
                    .email
                    .clone()
                    .unwrap()
                    .with_name(FOO.profile.display_name.clone().into_inner()),
            )
            .await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn subscribe_to_newsletter_ok() {
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

        let sut = UserEmailConfirmationServiceImpl {
            user_repo,
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .subscribe_to_newsletter(&mut (), FOO.user.id, VERIFICATION_CODE_1.clone())
            .await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn subscribe_to_newsletter_no_code_requested() {
        // Arrange
        let user_repo = MockUserRepository::new();

        let cache = MockCacheService::new().with_get(
            format!("subscribe_newsletter_code:{}", FOO.user.id.hyphenated()),
            None::<String>,
        );

        let sut = UserEmailConfirmationServiceImpl {
            user_repo,
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .subscribe_to_newsletter(&mut (), FOO.user.id, VERIFICATION_CODE_1.clone())
            .await;

        // Assert
        assert_matches!(
            result,
            Err(UserEmailConfirmationSubscribeToNewsletterError::InvalidCode)
        );
    }

    #[tokio::test]
    async fn subscribe_to_newsletter_invalid_code() {
        // Arrange
        let user_repo = MockUserRepository::new();

        let cache = MockCacheService::new().with_get(
            format!("subscribe_newsletter_code:{}", FOO.user.id.hyphenated()),
            VERIFICATION_CODE_2.clone().into_inner().into(),
        );

        let sut = UserEmailConfirmationServiceImpl {
            user_repo,
            cache,
            ..Sut::default()
        };

        // Act
        let result = sut
            .subscribe_to_newsletter(&mut (), FOO.user.id, VERIFICATION_CODE_1.clone())
            .await;

        // Assert
        assert_matches!(
            result,
            Err(UserEmailConfirmationSubscribeToNewsletterError::InvalidCode)
        );
    }
}
