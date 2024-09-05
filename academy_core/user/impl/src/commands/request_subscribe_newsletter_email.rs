use std::{sync::Arc, time::Duration};

use academy_cache_contracts::CacheService;
use academy_core_user_contracts::commands::request_subscribe_newsletter_email::UserRequestSubscribeNewsletterEmailCommandService;
use academy_di::Build;
use academy_email_contracts::template::TemplateEmailService;
use academy_models::{email_address::EmailAddressWithName, user::UserId};
use academy_shared_contracts::secret::SecretService;
use academy_templates_contracts::SubscribeNewsletterTemplate;

use crate::subscribe_newsletter_cache_key;

#[derive(Debug, Clone, Build)]
pub struct UserRequestSubscribeNewsletterEmailCommandServiceImpl<Secret, TemplateEmail, Cache> {
    secret: Secret,
    template_email: TemplateEmail,
    cache: Cache,
    config: UserRequestSubscribeNewsletterEmailCommandServiceConfig,
}

#[derive(Debug, Clone)]
pub struct UserRequestSubscribeNewsletterEmailCommandServiceConfig {
    pub redirect_url: Arc<String>,
    pub verification_code_ttl: Duration,
}

impl<Secret, TemplateEmail, Cache> UserRequestSubscribeNewsletterEmailCommandService
    for UserRequestSubscribeNewsletterEmailCommandServiceImpl<Secret, TemplateEmail, Cache>
where
    Secret: SecretService,
    TemplateEmail: TemplateEmailService,
    Cache: CacheService,
{
    async fn invoke(&self, user_id: UserId, email: EmailAddressWithName) -> anyhow::Result<()> {
        let code = self.secret.generate_verification_code();

        self.cache
            .set(
                &subscribe_newsletter_cache_key(user_id),
                &*code,
                Some(self.config.verification_code_ttl),
            )
            .await?;

        self.template_email
            .send_subscribe_newsletter_email(
                email,
                &SubscribeNewsletterTemplate {
                    code: code.into_inner(),
                    url: self.config.redirect_url.to_string(),
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
        let url = "https://bootstrap.academy/account/newsletter";

        let config = UserRequestSubscribeNewsletterEmailCommandServiceConfig {
            redirect_url: Arc::new(url.into()),
            verification_code_ttl: Duration::from_secs(3600),
        };

        let secret =
            MockSecretService::new().with_generate_verification_code(VERIFICATION_CODE_1.clone());

        let expected_email = SubscribeNewsletterTemplate {
            code: VERIFICATION_CODE_1.clone().into_inner(),
            url: url.into(),
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
            Some(config.verification_code_ttl),
        );

        let sut = UserRequestSubscribeNewsletterEmailCommandServiceImpl {
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
