use std::{path::Path, sync::Arc};

use academy_config::{RecaptchaConfig, DEFAULT_CONFIG_PATH};
use academy_di::{provider, Provides};
use academy_extern_contracts::recaptcha::{RecaptchaApiService, RecaptchaSiteverifyResponse};
use academy_extern_impl::recaptcha::{RecaptchaApiServiceConfig, RecaptchaApiServiceImpl};

#[tokio::test]
async fn success_score() {
    let (sut, secret) = make_sut();
    let result = sut.siteverify("success-0.7", &secret).await.unwrap();
    assert_eq!(
        result,
        RecaptchaSiteverifyResponse {
            success: true,
            score: Some(0.7)
        }
    );
}

#[tokio::test]
async fn success_no_score() {
    let (sut, secret) = make_sut();
    let result = sut.siteverify("success", &secret).await.unwrap();
    assert_eq!(
        result,
        RecaptchaSiteverifyResponse {
            success: true,
            score: None
        }
    );
}

#[tokio::test]
async fn failure() {
    let (sut, secret) = make_sut();
    let result = sut.siteverify("failure", &secret).await.unwrap();
    assert_eq!(
        result,
        RecaptchaSiteverifyResponse {
            success: false,
            score: None
        }
    );
}

fn make_sut() -> (RecaptchaApiServiceImpl, String) {
    let paths = vec![Path::new(DEFAULT_CONFIG_PATH)];
    let config = academy_config::load_with_override(&paths, &["recaptcha.enable = true"]).unwrap();

    let RecaptchaConfig {
        siteverify_endpoint_override,
        secret,
        ..
    } = config.recaptcha.unwrap();

    provider! {
        Provider { recaptcha_api_service_config: Arc<RecaptchaApiServiceConfig>, }
    }

    let mut provider = Provider {
        _state: Default::default(),
        recaptcha_api_service_config: RecaptchaApiServiceConfig::new(siteverify_endpoint_override)
            .into(),
    };

    (provider.provide(), secret)
}
