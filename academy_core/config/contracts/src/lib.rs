#[cfg_attr(feature = "mock", mockall::automock)]
pub trait ConfigService: Send + Sync + 'static {
    #[allow(clippy::needless_lifetimes)]
    fn get_recaptcha_sitekey<'a>(&'a self) -> Option<&'a str>;
}
