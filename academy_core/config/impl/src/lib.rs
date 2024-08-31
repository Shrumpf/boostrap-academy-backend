use academy_core_config_contracts::ConfigService;
use academy_di::Build;
use academy_shared_contracts::captcha::CaptchaService;

#[derive(Debug, Clone, Build)]
pub struct ConfigServiceImpl<Captcha> {
    captcha: Captcha,
}

impl<Captcha> ConfigService for ConfigServiceImpl<Captcha>
where
    Captcha: CaptchaService,
{
    fn get_recaptcha_sitekey(&self) -> Option<&str> {
        self.captcha.get_recaptcha_sitekey()
    }
}

#[cfg(test)]
mod tests {
    use academy_shared_contracts::captcha::MockCaptchaService;

    use super::*;

    #[test]
    fn get_sitekey_enabled() {
        // Arrange
        let captcha = MockCaptchaService::new().with_get_recaptcha_sitekey(Some("sitekey"));

        let sut = ConfigServiceImpl { captcha };

        // Act
        let result = sut.get_recaptcha_sitekey();

        // Assert
        assert_eq!(result, Some("sitekey"));
    }

    #[test]
    fn get_sitekey_disabled() {
        // Arrange
        let captcha = MockCaptchaService::new().with_get_recaptcha_sitekey(None);

        let sut = ConfigServiceImpl { captcha };

        // Act
        let result = sut.get_recaptcha_sitekey();

        // Assert
        assert_eq!(result, None);
    }
}
