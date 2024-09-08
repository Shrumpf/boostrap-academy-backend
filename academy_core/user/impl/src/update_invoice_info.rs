use academy_core_user_contracts::update_invoice_info::UserUpdateInvoiceInfoService;
use academy_di::Build;
use academy_models::user::{UserId, UserInvoiceInfo, UserInvoiceInfoPatch};
use academy_persistence_contracts::user::UserRepository;
use academy_utils::patch::{Patch, PatchValue};

#[derive(Debug, Clone, Build)]
pub struct UserUpdateInvoiceInfoServiceImpl<UserRepo> {
    user_repo: UserRepo,
}

impl<Txn, UserRepo> UserUpdateInvoiceInfoService<Txn> for UserUpdateInvoiceInfoServiceImpl<UserRepo>
where
    Txn: Send + Sync + 'static,
    UserRepo: UserRepository<Txn>,
{
    async fn invoke(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        invoice_info: UserInvoiceInfo,
        mut patch: UserInvoiceInfoPatch,
    ) -> anyhow::Result<UserInvoiceInfo> {
        if patch.business.update(invoice_info.business) == Some(false) {
            patch.vat_id = PatchValue::Update(None).minimize(&invoice_info.vat_id);
        }

        self.user_repo
            .update_invoice_info(txn, user_id, patch.as_ref())
            .await?;

        Ok(invoice_info.update(patch))
    }
}

#[cfg(test)]
mod tests {
    use academy_demo::user::{BAR, FOO};
    use academy_persistence_contracts::user::MockUserRepository;
    use academy_utils::{patch::Patch, Apply};

    use super::*;

    #[tokio::test]
    async fn ok() {
        // Arrange
        let expected = FOO.invoice_info.clone();
        let patch = FOO.invoice_info.clone().into_patch();

        let user_repo =
            MockUserRepository::new().with_update_invoice_info(BAR.user.id, patch.clone(), true);

        let sut = UserUpdateInvoiceInfoServiceImpl { user_repo };

        // Act
        let result = sut
            .invoke(&mut (), BAR.user.id, BAR.invoice_info.clone(), patch)
            .await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn reset_vat_id_if_not_business() {
        // Arrange
        let expected = FOO.invoice_info.clone().with(|u| {
            u.business = Some(false);
            u.vat_id = None;
        });
        let patch = UserInvoiceInfoPatch::new().update_business(Some(false));

        let user_repo = MockUserRepository::new().with_update_invoice_info(
            FOO.user.id,
            patch.clone().update_vat_id(None),
            true,
        );

        let sut = UserUpdateInvoiceInfoServiceImpl { user_repo };

        // Act
        let result = sut
            .invoke(&mut (), FOO.user.id, FOO.invoice_info.clone(), patch)
            .await;

        // Assert
        assert_eq!(result.unwrap(), expected);
    }
}
