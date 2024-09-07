use std::future::Future;

use academy_models::{auth::Login, session::DeviceName, user::UserComposite};

/// Creates a new session for a given user.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait SessionCreateCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        user_composite: UserComposite,
        device_name: Option<DeviceName>,
        update_last_login: bool,
    ) -> impl Future<Output = anyhow::Result<Login>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockSessionCreateCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        user_composite: UserComposite,
        device_name: Option<DeviceName>,
        update_last_login: bool,
        result: Login,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_composite),
                mockall::predicate::eq(device_name),
                mockall::predicate::eq(update_last_login),
            )
            .return_once(|_, _, _, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
