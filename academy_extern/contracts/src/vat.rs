use std::future::Future;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait VatApiService: Send + Sync + 'static {
    /// Validate the given VAT id.
    fn is_vat_id_valid(&self, vat_id: &str) -> impl Future<Output = anyhow::Result<bool>> + Send;
}

#[cfg(feature = "mock")]
impl MockVatApiService {
    pub fn with_is_vat_id_valid(mut self, vat_id: String, result: bool) -> Self {
        self.expect_is_vat_id_valid()
            .once()
            .with(mockall::predicate::eq(vat_id))
            .return_once(move |_| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
