use std::future::Future;

use academy_models::user::UserId;

/// Updates the `enabled` status of a user.
///
/// If `enabled` is changed to `false`, the user is automatically logged out of
/// all active sessions.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserUpdateEnabledCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        enabled: bool,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserUpdateEnabledCommandService<Txn> {
    pub fn with_invoke(mut self, user_id: UserId, enabled: bool, result: bool) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(enabled),
            )
            .return_once(move |_, _, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
