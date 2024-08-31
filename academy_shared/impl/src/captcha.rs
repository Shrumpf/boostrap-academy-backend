use academy_extern_contracts::recaptcha::RecaptchaApiService;
use academy_shared_contracts::captcha::{CaptchaCheckError, CaptchaService};

pub struct CaptchaServiceImpl<RecaptchaApi> {
    recaptcha_api: Option<RecaptchaApi>,
    config: CaptchaServiceConfig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaptchaServiceConfig {
    pub min_score: f64,
}

impl<RecaptchaApi> CaptchaService for CaptchaServiceImpl<RecaptchaApi>
where
    RecaptchaApi: RecaptchaApiService,
{
    async fn check(&self, response: Option<&str>) -> Result<(), CaptchaCheckError> {
        let Some(recaptcha_api) = &self.recaptcha_api else {
            return Ok(());
        };

        let response = response.ok_or(CaptchaCheckError::Failed)?;
        let response = recaptcha_api.siteverify(response).await?;
        let ok = response.success && response.score.unwrap_or(0.0) >= self.config.min_score;
        ok.then_some(()).ok_or(CaptchaCheckError::Failed)
    }
}

#[cfg(test)]
mod tests {
    use academy_extern_contracts::recaptcha::{
        MockRecaptchaApiService, RecaptchaSiteverifyResponse,
    };
    use academy_utils::assert_matches;

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let config = CaptchaServiceConfig { min_score: 0.5 };

        let recaptcha_api = Some(MockRecaptchaApiService::new().with_siteverify(
            "captcha response".into(),
            RecaptchaSiteverifyResponse {
                success: true,
                score: Some(0.7),
            },
        ));

        let sut = CaptchaServiceImpl {
            recaptcha_api,
            config,
        };

        // Act
        let result = sut.check(Some("captcha response")).await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn ok_no_score() {
        // Arrange
        let config = CaptchaServiceConfig { min_score: 0.0 };

        let recaptcha_api = Some(MockRecaptchaApiService::new().with_siteverify(
            "captcha response".into(),
            RecaptchaSiteverifyResponse {
                success: true,
                score: None,
            },
        ));

        let sut = CaptchaServiceImpl {
            recaptcha_api,
            config,
        };

        // Act
        let result = sut.check(Some("captcha response")).await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn ok_disabled() {
        // Arrange
        let config = CaptchaServiceConfig { min_score: 0.5 };

        let recaptcha_api = None::<MockRecaptchaApiService>;

        let sut = CaptchaServiceImpl {
            recaptcha_api,
            config,
        };

        // Act
        let result = sut.check(Some("captcha response")).await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn ok_disabled_no_response() {
        // Arrange
        let config = CaptchaServiceConfig { min_score: 0.5 };

        let recaptcha_api = None::<MockRecaptchaApiService>;

        let sut = CaptchaServiceImpl {
            recaptcha_api,
            config,
        };

        // Act
        let result = sut.check(None).await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn failed_no_score() {
        // Arrange
        let config = CaptchaServiceConfig { min_score: 0.1 };

        let recaptcha_api = Some(MockRecaptchaApiService::new().with_siteverify(
            "captcha response".into(),
            RecaptchaSiteverifyResponse {
                success: true,
                score: None,
            },
        ));

        let sut = CaptchaServiceImpl {
            recaptcha_api,
            config,
        };

        // Act
        let result = sut.check(Some("captcha response")).await;

        // Assert
        assert_matches!(result, Err(CaptchaCheckError::Failed));
    }

    #[tokio::test]
    async fn failed_insufficient_score() {
        // Arrange
        let config = CaptchaServiceConfig { min_score: 0.7 };

        let recaptcha_api = Some(MockRecaptchaApiService::new().with_siteverify(
            "captcha response".into(),
            RecaptchaSiteverifyResponse {
                success: true,
                score: Some(0.3),
            },
        ));

        let sut = CaptchaServiceImpl {
            recaptcha_api,
            config,
        };

        // Act
        let result = sut.check(Some("captcha response")).await;

        // Assert
        assert_matches!(result, Err(CaptchaCheckError::Failed));
    }

    #[tokio::test]
    async fn failed_no_success() {
        // Arrange
        let config = CaptchaServiceConfig { min_score: 0.5 };

        let recaptcha_api = Some(MockRecaptchaApiService::new().with_siteverify(
            "captcha response".into(),
            RecaptchaSiteverifyResponse {
                success: false,
                score: None,
            },
        ));

        let sut = CaptchaServiceImpl {
            recaptcha_api,
            config,
        };

        // Act
        let result = sut.check(Some("captcha response")).await;

        // Assert
        assert_matches!(result, Err(CaptchaCheckError::Failed));
    }

    #[tokio::test]
    async fn failed_no_response() {
        // Arrange
        let config = CaptchaServiceConfig { min_score: 0.5 };

        let recaptcha_api = Some(MockRecaptchaApiService::new());

        let sut = CaptchaServiceImpl {
            recaptcha_api,
            config,
        };

        // Act
        let result = sut.check(None).await;

        // Assert
        assert_matches!(result, Err(CaptchaCheckError::Failed));
    }
}
