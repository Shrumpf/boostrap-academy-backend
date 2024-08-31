use std::{path::Path, sync::Arc};

use academy_config::DEFAULT_CONFIG_PATH;
use academy_di::{provider, Provides};
use academy_extern_contracts::recaptcha::{RecaptchaApiService, RecaptchaSiteverifyResponse};
use academy_extern_impl::recaptcha::{RecaptchaApiServiceConfig, RecaptchaApiServiceImpl};

#[tokio::test]
async fn success_score() {
    let result = make_sut().siteverify("success-0.7").await.unwrap();
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
    let result = make_sut().siteverify("success").await.unwrap();
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
    let result = make_sut().siteverify("failure").await.unwrap();
    assert_eq!(
        result,
        RecaptchaSiteverifyResponse {
            success: false,
            score: None
        }
    );
}

fn make_sut() -> RecaptchaApiServiceImpl {
    let mut paths = vec![Path::new(DEFAULT_CONFIG_PATH)];
    let extra = std::env::var("EXTRA_CONFIG");
    if let Ok(extra) = &extra {
        paths.push(Path::new(extra));
    }
    let config = academy_config::load(&paths).unwrap();

    provider! {
        Provider { recaptcha_api_service_config: Arc<RecaptchaApiServiceConfig>, }
    }

    let mut provider = Provider {
        _state: Default::default(),
        recaptcha_api_service_config: RecaptchaApiServiceConfig {
            siteverify_endpoint: config.recaptcha.siteverify_endpoint,
            secret: config.recaptcha.secret,
        }
        .into(),
    };

    provider.provide()
}
