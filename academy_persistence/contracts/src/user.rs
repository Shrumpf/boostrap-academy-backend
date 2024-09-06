use std::future::Future;

use academy_models::{
    email_address::EmailAddress,
    oauth2::{OAuth2ProviderId, OAuth2RemoteUserId},
    pagination::PaginationSlice,
    user::{
        User, UserComposite, UserFilter, UserId, UserInvoiceInfo, UserInvoiceInfoPatchRef,
        UserName, UserPatchRef, UserProfile, UserProfilePatchRef,
    },
};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UserRepository<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    /// Return the number of users matching the given filter.
    fn count(
        &self,
        txn: &mut Txn,
        filter: &UserFilter,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;

    /// Returns all user composites matching the given filter and pagination
    /// slice.
    fn list_composites(
        &self,
        txn: &mut Txn,
        filter: &UserFilter,
        pagination: PaginationSlice,
    ) -> impl Future<Output = anyhow::Result<Vec<UserComposite>>> + Send;

    /// Returns whether the user with the given id exists.
    fn exists(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    /// Returns the user composite with the given id.
    fn get_composite(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<Option<UserComposite>>> + Send;

    /// Returns the user with the given name.
    fn get_composite_by_name(
        &self,
        txn: &mut Txn,
        name: &UserName,
    ) -> impl Future<Output = anyhow::Result<Option<UserComposite>>> + Send;

    /// Returns the user with the given email.
    fn get_composite_by_email(
        &self,
        txn: &mut Txn,
        email: &EmailAddress,
    ) -> impl Future<Output = anyhow::Result<Option<UserComposite>>> + Send;

    /// Returns the user linked to the given remote oauth2 user.
    fn get_composite_by_oauth2_provider_id_and_remote_user_id(
        &self,
        txn: &mut Txn,
        provider_id: &OAuth2ProviderId,
        remote_user_id: &OAuth2RemoteUserId,
    ) -> impl Future<Output = anyhow::Result<Option<UserComposite>>> + Send;

    /// Creates a new user.
    ///
    /// Returns an error if a user with the same name or email already exists
    /// (both case insensitive).
    fn create(
        &self,
        txn: &mut Txn,
        user: &User,
        profile: &UserProfile,
        invoice_info: &UserInvoiceInfo,
    ) -> impl Future<Output = Result<(), UserRepoError>> + Send;

    fn update<'a>(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        patch: UserPatchRef<'a>,
    ) -> impl Future<Output = Result<bool, UserRepoError>> + Send;

    fn update_profile<'a>(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        patch: UserProfilePatchRef<'a>,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    fn update_invoice_info<'a>(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        patch: UserInvoiceInfoPatchRef<'a>,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    fn delete(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    /// Saves or updates the password hash for a given user.
    fn save_password_hash(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        password_hash: String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Returns the password hash of a given user.
    fn get_password_hash(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<Option<String>>> + Send;

    /// Removes the password hash of a given user.
    fn remove_password_hash(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;
}

#[derive(Debug, Error)]
pub enum UserRepoError {
    #[error("A user with the same name already exists.")]
    NameConflict,
    #[error("A user with the same email address already exists.")]
    EmailConflict,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockUserRepository<Txn> {
    pub fn with_count(mut self, filter: UserFilter, result: u64) -> Self {
        self.expect_count()
            .once()
            .with(mockall::predicate::always(), mockall::predicate::eq(filter))
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_list_composites(
        mut self,
        filter: UserFilter,
        pagination: PaginationSlice,
        result: Vec<UserComposite>,
    ) -> Self {
        self.expect_list_composites()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(filter),
                mockall::predicate::eq(pagination),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_exists(mut self, user_id: UserId, result: bool) -> Self {
        self.expect_exists()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_get_composite(mut self, user_id: UserId, result: Option<UserComposite>) -> Self {
        self.expect_get_composite()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_get_composite_by_name(
        mut self,
        name: UserName,
        result: Option<UserComposite>,
    ) -> Self {
        self.expect_get_composite_by_name()
            .once()
            .with(mockall::predicate::always(), mockall::predicate::eq(name))
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_get_composite_by_email(
        mut self,
        email: EmailAddress,
        result: Option<UserComposite>,
    ) -> Self {
        self.expect_get_composite_by_email()
            .once()
            .with(mockall::predicate::always(), mockall::predicate::eq(email))
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_get_composite_by_oauth2_provider_id_and_remote_user_id(
        mut self,
        provider_id: OAuth2ProviderId,
        remote_user_id: OAuth2RemoteUserId,
        result: Option<UserComposite>,
    ) -> Self {
        self.expect_get_composite_by_oauth2_provider_id_and_remote_user_id()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(provider_id),
                mockall::predicate::eq(remote_user_id),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_create(
        mut self,
        user: User,
        profile: UserProfile,
        invoice_info: UserInvoiceInfo,
        result: Result<(), UserRepoError>,
    ) -> Self {
        self.expect_create()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user),
                mockall::predicate::eq(profile),
                mockall::predicate::eq(invoice_info),
            )
            .return_once(|_, _, _, _| Box::pin(std::future::ready(result)));
        self
    }

    pub fn with_update(
        mut self,
        user_id: UserId,
        patch: academy_models::user::UserPatch,
        result: Result<bool, UserRepoError>,
    ) -> Self {
        self.expect_update()
            .once()
            .withf(move |_, id, p| *id == user_id && p == &patch.as_ref())
            .return_once(|_, _, _| Box::pin(std::future::ready(result)));
        self
    }

    pub fn with_update_profile(
        mut self,
        user_id: UserId,
        patch: academy_models::user::UserProfilePatch,
        result: bool,
    ) -> Self {
        self.expect_update_profile()
            .once()
            .withf(move |_, id, p| *id == user_id && p == &patch.as_ref())
            .return_once(move |_, _, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_delete(mut self, user_id: UserId, result: bool) -> Self {
        self.expect_delete()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_save_password_hash(mut self, user_id: UserId, password_hash: String) -> Self {
        self.expect_save_password_hash()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(password_hash),
            )
            .return_once(|_, _, _| Box::pin(std::future::ready(Ok(()))));
        self
    }

    pub fn with_get_password_hash(
        mut self,
        user_id: UserId,
        password_hash: Option<String>,
    ) -> Self {
        self.expect_get_password_hash()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(password_hash))));
        self
    }

    pub fn with_remove_password_hash(mut self, user_id: UserId, result: bool) -> Self {
        self.expect_remove_password_hash()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
