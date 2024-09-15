use academy_auth_contracts::{
    AuthResultExt, AuthService, AuthenticateByPasswordError, AuthenticateByRefreshTokenError,
};
use academy_core_mfa_contracts::authenticate::{
    MfaAuthenticateError, MfaAuthenticateResult, MfaAuthenticateService,
};
use academy_core_session_contracts::{
    commands::{
        create::SessionCreateCommandService,
        delete::SessionDeleteCommandService,
        delete_by_user::SessionDeleteByUserCommandService,
        refresh::{SessionRefreshCommandError, SessionRefreshCommandService},
    },
    failed_auth_count::SessionFailedAuthCountService,
    SessionCreateCommand, SessionCreateError, SessionDeleteByUserError, SessionDeleteCurrentError,
    SessionDeleteError, SessionFeatureService, SessionGetCurrentError, SessionImpersonateError,
    SessionListByUserError, SessionRefreshError,
};
use academy_core_user_contracts::queries::get_by_name_or_email::UserGetByNameOrEmailQueryService;
use academy_di::Build;
use academy_models::{
    auth::Login,
    session::{Session, SessionId},
    user::{UserId, UserIdOrSelf, UserNameOrEmailAddress},
    RecaptchaResponse,
};
use academy_persistence_contracts::{
    session::SessionRepository, user::UserRepository, Database, Transaction,
};
use academy_shared_contracts::captcha::{CaptchaCheckError, CaptchaService};
use anyhow::anyhow;

pub mod commands;
pub mod failed_auth_count;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Build)]
#[cfg_attr(test, derive(Default))]
pub struct SessionFeatureServiceImpl<
    Db,
    Auth,
    Captcha,
    SessionCreate,
    SessionRefresh,
    SessionDelete,
    SessionDeleteByUser,
    SessionFailedAuthCount,
    UserGetByNameOrEmail,
    MfaAuthenticate,
    UserRepo,
    SessionRepo,
> {
    db: Db,
    auth: Auth,
    captcha: Captcha,
    session_create: SessionCreate,
    session_refresh: SessionRefresh,
    session_delete: SessionDelete,
    session_delete_by_user: SessionDeleteByUser,
    session_failed_auth_count: SessionFailedAuthCount,
    user_get_by_name_or_email: UserGetByNameOrEmail,
    mfa_authenticate: MfaAuthenticate,
    user_repo: UserRepo,
    session_repo: SessionRepo,
    config: SessionServiceConfig,
}

#[derive(Debug, Clone)]
pub struct SessionServiceConfig {
    pub login_fails_before_captcha: u64,
}

impl<
        Db,
        Auth,
        Captcha,
        SessionCreate,
        SessionRefresh,
        SessionDelete,
        SessionDeleteByUser,
        SessionFailedAuthCount,
        UserGetByNameOrEmail,
        MfaAuthenticate,
        UserRepo,
        SessionRepo,
    > SessionFeatureService
    for SessionFeatureServiceImpl<
        Db,
        Auth,
        Captcha,
        SessionCreate,
        SessionRefresh,
        SessionDelete,
        SessionDeleteByUser,
        SessionFailedAuthCount,
        UserGetByNameOrEmail,
        MfaAuthenticate,
        UserRepo,
        SessionRepo,
    >
where
    Db: Database,
    Auth: AuthService<Db::Transaction>,
    Captcha: CaptchaService,
    SessionCreate: SessionCreateCommandService<Db::Transaction>,
    SessionRefresh: SessionRefreshCommandService<Db::Transaction>,
    SessionDelete: SessionDeleteCommandService<Db::Transaction>,
    SessionDeleteByUser: SessionDeleteByUserCommandService<Db::Transaction>,
    SessionFailedAuthCount: SessionFailedAuthCountService,
    UserGetByNameOrEmail: UserGetByNameOrEmailQueryService<Db::Transaction>,
    MfaAuthenticate: MfaAuthenticateService<Db::Transaction>,
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

    async fn create_session(
        &self,
        cmd: SessionCreateCommand,
        recaptcha_response: Option<RecaptchaResponse>,
    ) -> Result<Login, SessionCreateError> {
        let failed_login_attempts = self
            .session_failed_auth_count
            .get(&cmd.name_or_email)
            .await?;

        if failed_login_attempts >= self.config.login_fails_before_captcha {
            self.captcha
                .check(recaptcha_response.as_deref().map(String::as_str))
                .await
                .map_err(|err| match err {
                    CaptchaCheckError::Failed => SessionCreateError::Recaptcha,
                    CaptchaCheckError::Other(err) => err.into(),
                })?;
        }

        let mut txn = self.db.begin_transaction().await?;

        let mut user_composite = match self
            .user_get_by_name_or_email
            .invoke(&mut txn, &cmd.name_or_email)
            .await?
        {
            Some(user_composite) => user_composite,
            None => {
                self.session_failed_auth_count
                    .increment(&cmd.name_or_email)
                    .await?;
                return Err(SessionCreateError::InvalidCredentials);
            }
        };

        let increment_failed_login_attempts = || async {
            self.session_failed_auth_count
                .increment(&UserNameOrEmailAddress::Name(
                    user_composite.user.name.clone(),
                ))
                .await?;
            if let Some(email) = user_composite.user.email.clone() {
                self.session_failed_auth_count
                    .increment(&UserNameOrEmailAddress::Email(email))
                    .await?;
            }
            anyhow::Ok(())
        };

        match self
            .auth
            .authenticate_by_password(&mut txn, user_composite.user.id, cmd.password)
            .await
        {
            Ok(()) => {}
            Err(AuthenticateByPasswordError::InvalidCredentials) => {
                increment_failed_login_attempts().await?;
                return Err(SessionCreateError::InvalidCredentials);
            }
            Err(AuthenticateByPasswordError::Other(err)) => return Err(err.into()),
        };

        if user_composite.details.mfa_enabled {
            match self
                .mfa_authenticate
                .authenticate(&mut txn, user_composite.user.id, cmd.mfa)
                .await
            {
                Ok(MfaAuthenticateResult::Ok) => (),
                Ok(MfaAuthenticateResult::Reset) => user_composite.details.mfa_enabled = false,
                Err(MfaAuthenticateError::Failed) => {
                    increment_failed_login_attempts().await?;
                    return Err(SessionCreateError::MfaFailed);
                }
                Err(MfaAuthenticateError::Other(err)) => return Err(err.into()),
            }
        }

        self.session_failed_auth_count
            .reset(&UserNameOrEmailAddress::Name(
                user_composite.user.name.clone(),
            ))
            .await?;
        if let Some(email) = user_composite.user.email.clone() {
            self.session_failed_auth_count
                .reset(&UserNameOrEmailAddress::Email(email))
                .await?;
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
