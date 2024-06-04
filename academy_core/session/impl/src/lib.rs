use academy_core_auth_contracts::{
    AuthResultExt, AuthService, AuthenticateByPasswordError, AuthenticateByRefreshTokenError,
};
use academy_core_mfa_contracts::commands::authenticate::{
    MfaAuthenticateCommandError, MfaAuthenticateCommandResult, MfaAuthenticateCommandService,
};
use academy_core_session_contracts::{
    commands::{
        create::SessionCreateCommandService,
        delete::SessionDeleteCommandService,
        delete_by_user::SessionDeleteByUserCommandService,
        refresh::{SessionRefreshCommandError, SessionRefreshCommandService},
    },
    SessionCreateCommand, SessionCreateError, SessionDeleteByUserError, SessionDeleteCurrentError,
    SessionDeleteError, SessionGetCurrentError, SessionImpersonateError, SessionListByUserError,
    SessionRefreshError, SessionService,
};
use academy_core_user_contracts::queries::get_by_name_or_email::UserGetByNameOrEmailQueryService;
use academy_di::Build;
use academy_models::{
    auth::Login,
    session::{Session, SessionId},
    user::{UserId, UserIdOrSelf},
};
use academy_persistence_contracts::{
    session::SessionRepository, user::UserRepository, Database, Transaction,
};
use anyhow::anyhow;

pub mod commands;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, Build)]
pub struct SessionServiceImpl<
    Db,
    Auth,
    SessionCreate,
    SessionRefresh,
    SessionDelete,
    SessionDeleteByUser,
    UserGetByNameOrEmail,
    MfaAuthenticate,
    UserRepo,
    SessionRepo,
> {
    db: Db,
    auth: Auth,
    session_create: SessionCreate,
    session_refresh: SessionRefresh,
    session_delete: SessionDelete,
    session_delete_by_user: SessionDeleteByUser,
    user_get_by_name_or_email: UserGetByNameOrEmail,
    mfa_authenticate: MfaAuthenticate,
    user_repo: UserRepo,
    session_repo: SessionRepo,
}

impl<
        Db,
        Auth,
        SessionCreate,
        SessionRefresh,
        SessionDelete,
        SessionDeleteByUser,
        UserGetByNameOrEmail,
        MfaAuthenticate,
        UserRepo,
        SessionRepo,
    > SessionService
    for SessionServiceImpl<
        Db,
        Auth,
        SessionCreate,
        SessionRefresh,
        SessionDelete,
        SessionDeleteByUser,
        UserGetByNameOrEmail,
        MfaAuthenticate,
        UserRepo,
        SessionRepo,
    >
where
    Db: Database,
    Auth: AuthService<Db::Transaction>,
    SessionCreate: SessionCreateCommandService<Db::Transaction>,
    SessionRefresh: SessionRefreshCommandService<Db::Transaction>,
    SessionDelete: SessionDeleteCommandService<Db::Transaction>,
    SessionDeleteByUser: SessionDeleteByUserCommandService<Db::Transaction>,
    UserGetByNameOrEmail: UserGetByNameOrEmailQueryService<Db::Transaction>,
    MfaAuthenticate: MfaAuthenticateCommandService<Db::Transaction>,
    UserRepo: UserRepository<Db::Transaction>,
    SessionRepo: SessionRepository<Db::Transaction>,
{
    async fn get_current_session(&self, token: &str) -> Result<Session, SessionGetCurrentError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        self.session_repo
            .get(&mut txn, auth.session_id)
            .await?
            .ok_or_else(|| anyhow!("Failed to get authenticated session").into())
    }

    async fn list_by_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> Result<Vec<Session>, SessionListByUserError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        self.session_repo
            .list_by_user(&mut txn, user_id)
            .await
            .map_err(Into::into)
    }

    async fn create_session(&self, cmd: SessionCreateCommand) -> Result<Login, SessionCreateError> {
        let mut txn = self.db.begin_transaction().await?;

        let mut user_composite = self
            .user_get_by_name_or_email
            .invoke(&mut txn, &cmd.name_or_email)
            .await?
            .ok_or(SessionCreateError::InvalidCredentials)?;

        self.auth
            .authenticate_by_password(&mut txn, user_composite.user.id, cmd.password)
            .await
            .map_err(|err| match err {
                AuthenticateByPasswordError::InvalidCredentials => {
                    SessionCreateError::InvalidCredentials
                }
                AuthenticateByPasswordError::Other(err) => err.into(),
            })?;

        if user_composite.details.mfa_enabled {
            match self
                .mfa_authenticate
                .invoke(&mut txn, user_composite.user.id, cmd.mfa)
                .await
                .map_err(|err| match err {
                    MfaAuthenticateCommandError::Failed => SessionCreateError::MfaFailed,
                    MfaAuthenticateCommandError::Other(err) => err.into(),
                })? {
                MfaAuthenticateCommandResult::Ok => (),
                MfaAuthenticateCommandResult::Reset => user_composite.details.mfa_enabled = false,
            }
        }

        if !user_composite.user.enabled {
            return Err(SessionCreateError::UserDisabled);
        }

        let login = self
            .session_create
            .invoke(&mut txn, user_composite, cmd.device_name, true)
            .await?;

        txn.commit().await?;

        Ok(login)
    }

    async fn impersonate(
        &self,
        token: &str,
        user_id: UserId,
    ) -> Result<Login, SessionImpersonateError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        auth.ensure_admin().map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        let user_composite = self
            .user_repo
            .get_composite(&mut txn, user_id)
            .await?
            .ok_or(SessionImpersonateError::NotFound)?;

        let login = self
            .session_create
            .invoke(&mut txn, user_composite, None, false)
            .await?;

        txn.commit().await?;

        Ok(login)
    }

    async fn refresh_session(&self, refresh_token: &str) -> Result<Login, SessionRefreshError> {
        let mut txn = self.db.begin_transaction().await?;

        let session_id = match self
            .auth
            .authenticate_by_refresh_token(&mut txn, refresh_token)
            .await
        {
            Ok(session_id) => session_id,
            Err(AuthenticateByRefreshTokenError::Invalid) => {
                return Err(SessionRefreshError::InvalidRefreshToken)
            }
            Err(AuthenticateByRefreshTokenError::Expired(session_id)) => {
                self.session_delete.invoke(&mut txn, session_id).await?;
                return Err(SessionRefreshError::InvalidRefreshToken);
            }
            Err(AuthenticateByRefreshTokenError::Other(err)) => return Err(err.into()),
        };

        let login = self
            .session_refresh
            .invoke(&mut txn, session_id)
            .await
            .map_err(|err| match err {
                SessionRefreshCommandError::NotFound => SessionRefreshError::InvalidRefreshToken,
                SessionRefreshCommandError::Other(err) => err.into(),
            })?;

        txn.commit().await?;

        Ok(login)
    }

    async fn delete_session(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
        session_id: SessionId,
    ) -> Result<(), SessionDeleteError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        let session = self
            .session_repo
            .get(&mut txn, session_id)
            .await?
            .filter(|s| s.user_id == user_id)
            .ok_or(SessionDeleteError::NotFound)?;

        self.session_delete.invoke(&mut txn, session.id).await?;

        txn.commit().await?;

        Ok(())
    }

    async fn delete_current_session(&self, token: &str) -> Result<(), SessionDeleteCurrentError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        self.session_delete
            .invoke(&mut txn, auth.session_id)
            .await?;

        txn.commit().await?;

        Ok(())
    }

    async fn delete_by_user(
        &self,
        token: &str,
        user_id: UserIdOrSelf,
    ) -> Result<(), SessionDeleteByUserError> {
        let auth = self.auth.authenticate(token).await.map_auth_err()?;
        let user_id = user_id.unwrap_or(auth.user_id);
        auth.ensure_self_or_admin(user_id).map_auth_err()?;

        let mut txn = self.db.begin_transaction().await?;

        self.session_delete_by_user
            .invoke(&mut txn, user_id)
            .await?;

        txn.commit().await?;

        Ok(())
    }
}
