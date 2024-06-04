use academy_config::EmailConfig;
use academy_email_impl::EmailServiceImpl;

pub async fn connect(config: &EmailConfig) -> anyhow::Result<EmailServiceImpl> {
    EmailServiceImpl::new(&config.smtp_url, config.from.clone()).await
}
