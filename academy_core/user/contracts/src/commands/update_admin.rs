use std::future::Future;

use academy_models::user::UserId;

/// Updates the `admin` status of a user.
///
/// Also invalidates any access token associated with this user, because they
/// also contain this field.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserUpdateAdminCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        admin: bool,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserUpdateAdminCommandService<Txn> {
    pub fn with_invoke(mut self, user_id: UserId, admin: bool, result: bool) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(admin),
            )
            .return_once(move |_, _, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
