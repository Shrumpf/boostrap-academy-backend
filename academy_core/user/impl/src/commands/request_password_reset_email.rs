use std::{sync::Arc, time::Duration};

use academy_cache_contracts::CacheService;
use academy_core_user_contracts::commands::request_password_reset_email::UserRequestPasswordResetEmailCommandService;
use academy_di::Build;
use academy_email_contracts::template::TemplateEmailService;
use academy_models::{email_address::EmailAddressWithName, user::UserId};
use academy_shared_contracts::secret::SecretService;
use academy_templates_contracts::ResetPasswordTemplate;

use crate::reset_password_cache_key;

#[derive(Debug, Clone, Build)]
pub struct UserRequestPasswordResetEmailCommandServiceImpl<Secret, TemplateEmail, Cache> {
    secret: Secret,
    template_email: TemplateEmail,
    cache: Cache,
    config: UserRequestPasswordResetEmailCommandServiceConfig,
}

#[derive(Debug, Clone)]
pub struct UserRequestPasswordResetEmailCommandServiceConfig {
    pub redirect_url: Arc<String>,
    pub verification_code_ttl: Duration,
}

impl<Secret, TemplateEmail, Cache> UserRequestPasswordResetEmailCommandService
    for UserRequestPasswordResetEmailCommandServiceImpl<Secret, TemplateEmail, Cache>
where
    Secret: SecretService,
    TemplateEmail: TemplateEmailService,
    Cache: CacheService,
{
    async fn invoke(&self, user_id: UserId, email: EmailAddressWithName) -> anyhow::Result<()> {
        let code = self.secret.generate_verification_code();

        self.cache
            .set(
                &reset_password_cache_key(user_id),
                &code,
                Some(self.config.verification_code_ttl),
            )
            .await?;

        self.template_email
            .send_reset_password_email(
                email,
                &ResetPasswordTemplate {
                    code: code.into_inner(),
                    url: (*self.config.redirect_url).clone(),
                },
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use academy_cache_contracts::MockCacheService;
    use academy_demo::{user::FOO, VERIFICATION_CODE_1};
    use academy_email_contracts::template::MockTemplateEmailService;
    use academy_shared_contracts::secret::MockSecretService;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let url = "https://bootstrap.academy/auth/reset-password";

        let config = UserRequestPasswordResetEmailCommandServiceConfig {
            redirect_url: url.to_owned().into(),
            verification_code_ttl: Duration::from_secs(3600),
        };

        let secret =
            MockSecretService::new().with_generate_verification_code(VERIFICATION_CODE_1.clone());

        let expected_email = ResetPasswordTemplate {
            code: VERIFICATION_CODE_1.clone().into_inner(),
            url: url.into(),
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
            Some(config.verification_code_ttl),
        );

        let sut = UserRequestPasswordResetEmailCommandServiceImpl {
            secret,
            template_email,
            cache,
            config,
        };

        // Act
        let result = sut
            .invoke(
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
}
