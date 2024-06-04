use std::future::Future;

use academy_models::user::{UserComposite, UserNameOrEmailAddress};

/// Returns the user with the given name or email.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserGetByNameOrEmailQueryService<Txn: Send + Sync + 'static>:
    Send + Sync + 'static
{
    fn invoke(
        &self,
        txn: &mut Txn,
        name_or_email: &UserNameOrEmailAddress,
    ) -> impl Future<Output = anyhow::Result<Option<UserComposite>>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserGetByNameOrEmailQueryService<Txn> {
    pub fn with_invoke(
        mut self,
        name_or_email: UserNameOrEmailAddress,
        result: Option<UserComposite>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(name_or_email),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
