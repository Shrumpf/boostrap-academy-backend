use academy_core_config_contracts::ConfigFeatureService;
use academy_di::Build;
use academy_shared_contracts::captcha::CaptchaService;
use academy_utils::trace_instrument;

#[derive(Debug, Clone, Build)]
pub struct ConfigFeatureServiceImpl<Captcha> {
    captcha: Captcha,
}

impl<Captcha> ConfigFeatureService for ConfigFeatureServiceImpl<Captcha>
where
    Captcha: CaptchaService,
{
    #[trace_instrument(skip(self))]
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

        let sut = ConfigFeatureServiceImpl { captcha };

        // Act
        let result = sut.get_recaptcha_sitekey();

        // Assert
        assert_eq!(result, Some("sitekey"));
    }

    #[test]
    fn get_sitekey_disabled() {
        // Arrange
        let captcha = MockCaptchaService::new().with_get_recaptcha_sitekey(None);

        let sut = ConfigFeatureServiceImpl { captcha };

        // Act
        let result = sut.get_recaptcha_sitekey();

        // Assert
        assert_eq!(result, None);
    }
}
