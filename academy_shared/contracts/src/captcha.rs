use std::future::Future;

use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait CaptchaService: Send + Sync + 'static {
    fn check<'a>(
        &self,
        response: Option<&'a str>,
    ) -> impl Future<Output = Result<(), CaptchaCheckError>> + Send;
}

#[derive(Debug, Error)]
pub enum CaptchaCheckError {
    #[error("The response is invalid or the user is probably not human.")]
    Failed,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl MockCaptchaService {
    pub fn with_check(
        mut self,
        response: Option<&'static str>,
        result: Result<(), CaptchaCheckError>,
    ) -> Self {
        self.expect_check()
            .once()
            .withf(move |x| *x == response)
            .return_once(|_| Box::pin(std::future::ready(result)));
        self
    }
}
