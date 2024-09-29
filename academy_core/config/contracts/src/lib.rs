pub trait ConfigFeatureService: Send + Sync + 'static {
    /// Return the public reCAPTCHA sitekey if reCAPTCHA is enabled.
    fn get_recaptcha_sitekey(&self) -> Option<&str>;
}
