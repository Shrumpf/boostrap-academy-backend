use std::future::Future;

use academy_models::{auth::Login, session::SessionId};
use thiserror::Error;

/// Refreshes a given session by issuing a new access/refresh token pair.
///
/// The old access and refresh tokens are invalidated.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait SessionRefreshCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        session_id: SessionId,
    ) -> impl Future<Output = Result<Login, SessionRefreshCommandError>> + Send;
}

#[derive(Debug, Error)]
pub enum SessionRefreshCommandError {
    #[error("The session does not exist.")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockSessionRefreshCommandService<Txn> {
    pub fn with_invoke(
        mut self,
        session_id: SessionId,
        result: Result<Login, SessionRefreshCommandError>,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(session_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(result)));
        self
    }
}
