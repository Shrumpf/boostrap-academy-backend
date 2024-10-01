use academy_auth_contracts::internal::AuthInternalService;
use academy_core_internal_contracts::{
    InternalGetUserByEmailError, InternalGetUserError, InternalService,
};
use academy_di::Build;
use academy_models::{
    email_address::EmailAddress,
    user::{UserComposite, UserId},
};
use academy_persistence_contracts::{user::UserRepository, Database};
use anyhow::Context;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Build, Default)]
pub struct InternalServiceImpl<Db, AuthInternal, UserRepo> {
    db: Db,
    auth_internal: AuthInternal,
    user_repo: UserRepo,
}

impl<Db, AuthInternal, UserRepo> InternalService for InternalServiceImpl<Db, AuthInternal, UserRepo>
where
    Db: Database,
    AuthInternal: AuthInternalService,
    UserRepo: UserRepository<Db::Transaction>,
{
    async fn get_user(
        &self,
        token: &str,
        user_id: UserId,
    ) -> Result<UserComposite, InternalGetUserError> {
        self.auth_internal.authenticate(token, "auth")?;

        let mut txn = self.db.begin_transaction().await?;

        self.user_repo
            .get_composite(&mut txn, user_id)
            .await
            .context("Failed to get user from database")?
            .ok_or(InternalGetUserError::NotFound)
    }

    async fn get_user_by_email(
        &self,
        token: &str,
        email: EmailAddress,
    ) -> Result<UserComposite, InternalGetUserByEmailError> {
        self.auth_internal.authenticate(token, "auth")?;

        let mut txn = self.db.begin_transaction().await?;

        self.user_repo
            .get_composite_by_email(&mut txn, &email)
            .await
            .context("Failed to get user from database")?
            .ok_or(InternalGetUserByEmailError::NotFound)
    }
}
