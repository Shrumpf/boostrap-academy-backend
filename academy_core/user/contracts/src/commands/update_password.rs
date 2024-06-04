use std::future::Future;

use academy_models::user::{UserId, UserPassword};

/// Updates the password of an existing user.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserUpdatePasswordCommandService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        password: UserPassword,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserUpdatePasswordCommandService<Txn> {
    pub fn with_invoke(mut self, user_id: UserId, password: UserPassword) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(password),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(Ok(()))));
        self
    }
}
