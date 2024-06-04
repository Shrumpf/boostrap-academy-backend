use academy_di::Build;
use academy_email_contracts::{template::TemplateEmailService, ContentType, Email, EmailService};
use academy_templates_contracts::{
    ResetPasswordTemplate, SubscribeNewsletterTemplate, Template, TemplateService,
    VerifyEmailTemplate,
};
use email_address::EmailAddress;

#[derive(Debug, Clone, Build)]
pub struct TemplateEmailServiceImpl<Email, Template> {
    email: Email,
    template: Template,
}

impl<EmailS, Template> TemplateEmailService for TemplateEmailServiceImpl<EmailS, Template>
where
    EmailS: EmailService,
    Template: TemplateService,
{
    async fn send_reset_password_email(
        &self,
        recipient: EmailAddress,
        data: &ResetPasswordTemplate,
    ) -> anyhow::Result<bool> {
        self.send_email(recipient, data, "Passwort zurücksetzen - Bootstrap Academy")
            .await
    }

    async fn send_subscribe_newsletter_email(
        &self,
        recipient: EmailAddress,
        data: &SubscribeNewsletterTemplate,
    ) -> anyhow::Result<bool> {
        self.send_email(recipient, data, "Newsletter abonnieren - Bootstrap Academy")
            .await
    }

    async fn send_verification_email(
        &self,
        recipient: EmailAddress,
        data: &VerifyEmailTemplate,
    ) -> anyhow::Result<bool> {
        self.send_email(recipient, data, "Willkommen bei der Bootstrap Academy!")
            .await
    }
}

impl<EmailS, TemplateS> TemplateEmailServiceImpl<EmailS, TemplateS>
where
    EmailS: EmailService,
    TemplateS: TemplateService,
{
    async fn send_email<T: Template + 'static>(
        &self,
        recipient: EmailAddress,
        data: &T,
        subject: impl Into<String>,
    ) -> anyhow::Result<bool> {
        self.email
            .send(Email {
                recipient,
                subject: subject.into(),
                body: self.template.render(data)?,
                content_type: ContentType::Html,
                reply_to: None,
            })
            .await
    }
}
