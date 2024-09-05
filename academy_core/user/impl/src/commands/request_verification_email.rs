use std::{sync::Arc, time::Duration};

use academy_cache_contracts::CacheService;
use academy_core_user_contracts::commands::request_verification_email::UserRequestVerificationEmailCommandService;
use academy_di::Build;
use academy_email_contracts::template::TemplateEmailService;
use academy_models::email_address::EmailAddressWithName;
use academy_shared_contracts::secret::SecretService;
use academy_templates_contracts::VerifyEmailTemplate;

use crate::verification_cache_key;

#[derive(Debug, Clone, Build)]
pub struct UserRequestVerificationEmailCommandServiceImpl<Secret, TemplateEmail, Cache> {
    secret: Secret,
    template_email: TemplateEmail,
    cache: Cache,
    config: UserRequestVerificationEmailCommandServiceConfig,
}

#[derive(Debug, Clone)]
pub struct UserRequestVerificationEmailCommandServiceConfig {
    pub redirect_url: Arc<String>,
    pub verification_code_ttl: Duration,
}

impl<Secret, TemplateEmail, Cache> UserRequestVerificationEmailCommandService
    for UserRequestVerificationEmailCommandServiceImpl<Secret, TemplateEmail, Cache>
where
    Secret: SecretService,
    TemplateEmail: TemplateEmailService,
    Cache: CacheService,
{
    async fn invoke(&self, email: EmailAddressWithName) -> anyhow::Result<()> {
        let code = self.secret.generate_verification_code();

        self.cache
            .set(
                &verification_cache_key(&code),
                &email.clone().into_email_address(),
                Some(self.config.verification_code_ttl),
            )
            .await?;

        self.template_email
            .send_verification_email(
                email,
                &VerifyEmailTemplate {
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
    use academy_templates_contracts::VerifyEmailTemplate;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let config = UserRequestVerificationEmailCommandServiceConfig {
            redirect_url: Arc::new("https://bootstrap.academy/auth/verify-account".into()),
            verification_code_ttl: Duration::from_secs(1337),
        };

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
                url: (*config.redirect_url).clone(),
            },
            true,
        );

        let cache = MockCacheService::new().with_set(
            format!("verification:{}", **VERIFICATION_CODE_1),
            FOO.user.email.clone().unwrap(),
            Some(config.verification_code_ttl),
        );

        let sut = UserRequestVerificationEmailCommandServiceImpl {
            secret,
            template_email,
            cache,
            config,
        };

        // Act
        let result = sut.invoke(recipient).await;

        // Assert
        result.unwrap();
    }
}
