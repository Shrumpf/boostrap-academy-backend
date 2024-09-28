#[cfg_attr(feature = "mock", mockall::automock)]
pub trait ConfigFeatureService: Send + Sync + 'static {
    /// Return the public reCAPTCHA sitekey if reCAPTCHA is enabled.
    #[allow(
        clippy::needless_lifetimes,
        reason = "explicit lifetime needed for automock"
    )]
    fn get_recaptcha_sitekey<'a>(&'a self) -> Option<&'a str>;
}
