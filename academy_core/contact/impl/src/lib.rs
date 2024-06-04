use std::sync::Arc;

use academy_core_contact_contracts::{ContactSendMessageError, ContactService};
use academy_di::Build;
use academy_email_contracts::{ContentType, Email, EmailService};
use academy_models::contact::ContactMessage;
use email_address::EmailAddress;

#[derive(Debug, Clone, Build)]
pub struct ContactServiceImpl<Email> {
    email: Email,
    config: ContactServiceConfig,
}

#[derive(Debug, Clone)]
pub struct ContactServiceConfig {
    pub email: Arc<EmailAddress>,
}

impl<EmailS> ContactService for ContactServiceImpl<EmailS>
where
    EmailS: EmailService,
{
    async fn send_message(&self, message: ContactMessage) -> Result<(), ContactSendMessageError> {
        let email = Email {
            recipient: (*self.config.email).clone(),
            subject: format!("[Contact Form] {}", *message.subject),
            body: format!(
                "Message from {} ({}):\n\n{}",
                *message.author.name, message.author.email, *message.content
            ),
            content_type: ContentType::Text,
            reply_to: Some(message.author.email),
        };

        if !self.email.send(email).await? {
            return Err(ContactSendMessageError::Send);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use academy_email_contracts::MockEmailService;
    use academy_models::contact::ContactMessageAuthor;
    use academy_utils::assert_matches;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let config = ContactServiceConfig {
            email: Arc::new("contact@example.com".parse().unwrap()),
        };

        let email = MockEmailService::new().with_send(
            Email {
                recipient: (*config.email).clone(),
                subject: "[Contact Form] Test".into(),
                body: "Message from Max Mustermann (max.mustermann@example.de):\n\nHello World!"
                    .into(),
                content_type: ContentType::Text,
                reply_to: Some("max.mustermann@example.de".parse().unwrap()),
            },
            true,
        );

        let sut = ContactServiceImpl { email, config };

        // Act
        let result = sut
            .send_message(ContactMessage {
                author: ContactMessageAuthor {
                    name: "Max Mustermann".try_into().unwrap(),
                    email: "max.mustermann@example.de".parse().unwrap(),
                },
                subject: "Test".try_into().unwrap(),
                content: "Hello World!".try_into().unwrap(),
            })
            .await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn error() {
        // Arrange
        let config = ContactServiceConfig {
            email: Arc::new("contact@example.com".parse().unwrap()),
        };

        let email = MockEmailService::new().with_send(
            Email {
                recipient: (*config.email).clone(),
                subject: "[Contact Form] Test".into(),
                body: "Message from Max Mustermann (max.mustermann@example.de):\n\nHello World!"
                    .into(),
                content_type: ContentType::Text,
                reply_to: Some("max.mustermann@example.de".parse().unwrap()),
            },
            false,
        );

        let sut = ContactServiceImpl { email, config };

        // Act
        let result = sut
            .send_message(ContactMessage {
                author: ContactMessageAuthor {
                    name: "Max Mustermann".try_into().unwrap(),
                    email: "max.mustermann@example.de".parse().unwrap(),
                },
                subject: "Test".try_into().unwrap(),
                content: "Hello World!".try_into().unwrap(),
            })
            .await;

        // Assert
        assert_matches!(result, Err(ContactSendMessageError::Send));
    }
}
