use std::future::Future;

use academy_models::user::UserId;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait MfaDisableService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    /// Completely disable MFA for the given user by deleting all TOTP devices
    /// and invalidating the MFA recovery code.
    fn disable(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockMfaDisableService<Txn> {
    pub fn with_disable(mut self, user_id: UserId) -> Self {
        self.expect_disable()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
