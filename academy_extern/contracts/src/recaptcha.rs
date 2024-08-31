use std::future::Future;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait RecaptchaApiService: Send + Sync + 'static {
    fn siteverify(
        &self,
        response: &str,
    ) -> impl Future<Output = anyhow::Result<RecaptchaSiteverifyResponse>> + Send;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecaptchaSiteverifyResponse {
    pub success: bool,
    pub score: Option<f64>,
}

#[cfg(feature = "mock")]
impl MockRecaptchaApiService {
    pub fn with_siteverify(
        mut self,
        response: String,
        result: RecaptchaSiteverifyResponse,
    ) -> Self {
        self.expect_siteverify()
            .once()
            .with(mockall::predicate::eq(response))
            .return_once(move |_| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
