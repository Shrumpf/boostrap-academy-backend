use std::future::Future;

use academy_models::user::{User, UserName};
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Updates the name of a user.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserUpdateNameCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        user: User,
        name: UserName,
        rate_limit_policy: UserUpdateNameRateLimitPolicy,
    ) -> impl Future<Output = Result<User, UserUpdateNameCommandError>> + Send;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserUpdateNameRateLimitPolicy {
    Enforce,
    Bypass,
}

#[derive(Debug, Error)]
pub enum UserUpdateNameCommandError {
    #[error("The user cannot change their name until {until}.")]
    RateLimit { until: DateTime<Utc> },
    #[error("A user with the same name already exists.")]
    Conflict,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserUpdateNameCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        user: User,
        name: UserName,
        rate_limit_policy: UserUpdateNameRateLimitPolicy,
        result: Result<User, UserUpdateNameCommandError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user),
                mockall::predicate::eq(name),
                mockall::predicate::eq(rate_limit_policy),
            )
            .return_once(|_, _, _, _| Box::pin(std::future::ready(result)));
        self
    }
}
