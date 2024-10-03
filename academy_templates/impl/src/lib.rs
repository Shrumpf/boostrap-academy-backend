use std::sync::Arc;

use academy_di::Build;
use academy_templates_contracts::{Template, TemplateService, BASE_TEMPLATE, TEMPLATES};
use anyhow::Context;
use tera::Tera;

#[derive(Debug, Clone, Build)]
pub struct TemplateServiceImpl {
    #[di(default)]
    state: State,
}

#[derive(Debug, Clone)]
struct State(Arc<Tera>);

impl Default for State {
    fn default() -> Self {
        let mut tera = Tera::default();

        tera.add_raw_template("base", BASE_TEMPLATE).unwrap();

        for &(name, template) in TEMPLATES {
            tera.add_raw_template(name, template).unwrap();
        }

        Self(tera.into())
    }
}

impl TemplateService for TemplateServiceImpl {
    fn render<T: Template>(&self, template: &T) -> anyhow::Result<String> {
        let context = tera::Context::from_serialize(template)
            .with_context(|| format!("Failed to build tera context for template {}", T::NAME))?;

        self.state
            .0
            .render(T::NAME, &context)
            .with_context(|| format!("Failed to render template {}", T::NAME))
    }
}

#[cfg(test)]
mod tests {
    use academy_templates_contracts::{
        ResetPasswordTemplate, SubscribeNewsletterTemplate, VerifyEmailTemplate,
    };

    use super::*;

    #[test]
    fn reset_password() {
        test_template(ResetPasswordTemplate {
            code: "code".into(),
            url: "https://bootstrap.academy/".into(),
        });
    }

    #[test]
    fn verify_email() {
        test_template(VerifyEmailTemplate {
            code: "code".into(),
            url: "https://bootstrap.academy/".into(),
        });
    }

    #[test]
    fn subscribe_newsletter() {
        test_template(SubscribeNewsletterTemplate {
            code: "code".into(),
            url: "https://bootstrap.academy/".into(),
        });
    }

    fn test_template<T: Template + 'static>(template: T) {
        // Arrange
        let sut = TemplateServiceImpl {
            state: Default::default(),
        };

        // Act
        let result = sut.render(&template);

        // Assert
        result.unwrap();
    }
}
