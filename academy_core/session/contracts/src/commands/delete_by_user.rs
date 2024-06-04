use std::future::Future;

use academy_models::user::UserId;

/// Deletes all sessions of a given user and invalidates all access and
/// refresh tokens associated with them.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait SessionDeleteByUserCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockSessionDeleteByUserCommandService<Txn> {
    pub fn with_invoke(mut self, user_id: UserId) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
