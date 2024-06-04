use std::future::Future;

use academy_models::session::SessionId;

/// Deletes a session and invalidates its access and refresh tokens.
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait SessionDeleteCommandService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        session_id: SessionId,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockSessionDeleteCommandService<Txn> {
    pub fn with_invoke(mut self, session_id: SessionId, result: bool) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(session_id),
            )
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
