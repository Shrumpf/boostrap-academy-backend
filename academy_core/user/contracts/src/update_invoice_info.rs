use std::future::Future;

use academy_models::user::{UserId, UserInvoiceInfo, UserInvoiceInfoPatch};

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserUpdateInvoiceInfoService<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        invoice_info: UserInvoiceInfo,
        patch: UserInvoiceInfoPatch,
    ) -> impl Future<Output = anyhow::Result<UserInvoiceInfo>> + Send;
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserUpdateInvoiceInfoService<Txn> {
    pub fn with_invoke(
        mut self,
        user_id: UserId,
        invoice_info: UserInvoiceInfo,
        patch: UserInvoiceInfoPatch,
        result: UserInvoiceInfo,
    ) -> Self {
        self.expect_invoke()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(invoice_info),
                mockall::predicate::eq(patch),
            )
            .return_once(|_, _, _, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
