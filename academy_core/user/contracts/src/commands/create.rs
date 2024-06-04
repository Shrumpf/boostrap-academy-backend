use std::future::Future;

use academy_models::user::{UserComposite, UserDisplayName, UserName, UserPassword};
use email_address::EmailAddress;
use thiserror::Error;

/// Creates a new user.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserCreateCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        cmd: UserCreateCommand,
    ) -> impl Future<Output = Result<UserComposite, UserCreateCommandError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserCreateCommand {
    pub name: UserName,
    pub display_name: UserDisplayName,
    pub email: EmailAddress,
    pub password: UserPassword,
    pub admin: bool,
    pub enabled: bool,
    pub email_verified: bool,
}

#[derive(Debug, Error)]
pub enum UserCreateCommandError {
    #[error("A user with the same name already exists.")]
    NameConflict,
    #[error("A user with the same email address already exists.")]
    EmailConflict,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserCreateCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        cmd: UserCreateCommand,
        result: Result<UserComposite, UserCreateCommandError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(mockall::predicate::always(), mockall::predicate::eq(cmd))
            .return_once(|_, _| Box::pin(std::future::ready(result)));
        self
    }
}
