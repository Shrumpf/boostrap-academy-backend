use std::time::Duration;

use academy_cache_contracts::CacheService;
use academy_core_auth_contracts::{
    commands::invalidate_access_token::AuthInvalidateAccessTokenCommandService, AuthService,
    AuthenticateByPasswordError, AuthenticateByRefreshTokenError, Authentication, Tokens,
};
use academy_di::Build;
use academy_models::{
    auth::AuthenticateError,
    session::{SessionId, SessionRefreshTokenHash},
    user::{User, UserId, UserPassword},
};
use academy_persistence_contracts::{session::SessionRepository, user::UserRepository};
use academy_shared_contracts::{
    hash::HashService,
    jwt::JwtService,
    password::{PasswordService, PasswordVerifyError},
    secret::SecretService,
    time::TimeService,
};
use serde::{Deserialize, Serialize};

pub mod commands;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Build)]
#[cfg_attr(test, derive(Default))]
pub struct AuthServiceImpl<
    Jwt,
    Secret,
    Time,
    Hash,
    Password,
    UserRepo,
    SessionRepo,
    Cache,
    AuthInvalidateAccessToken,
> {
    jwt: Jwt,
    secret: Secret,
    time: Time,
    hash: Hash,
    password: Password,
    user_repo: UserRepo,
    session_repo: SessionRepo,
    cache: Cache,
    auth_invalidate_access_token: AuthInvalidateAccessToken,
    config: AuthServiceConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct AuthServiceConfig {
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub refresh_token_length: usize,
}

impl<
        Txn,
        Jwt,
        Secret,
        Time,
        Hash,
        Password,
        UserRepo,
        SessionRepo,
        Cache,
        AuthInvalidateAccessToken,
    > AuthService<Txn>
    for AuthServiceImpl<
        Jwt,
        Secret,
        Time,
        Hash,
        Password,
        UserRepo,
        SessionRepo,
        Cache,
        AuthInvalidateAccessToken,
    >
where
    Txn: Send + Sync + 'static,
    Jwt: JwtService,
    Secret: SecretService,
    Time: TimeService,
    Hash: HashService,
    Password: PasswordService,
    UserRepo: UserRepository<Txn>,
    SessionRepo: SessionRepository<Txn>,
    Cache: CacheService,
    AuthInvalidateAccessToken: AuthInvalidateAccessTokenCommandService,
{
    async fn authenticate(&self, token: &str) -> Result<Authentication, AuthenticateError> {
        let auth = self
            .jwt
            .verify::<Token>(token)
            .map(Authentication::from)
            .map_err(|_| AuthenticateError::InvalidToken)?;

        if let Some(()) = self
            .cache
            .get(&access_token_invalidated_key(auth.refresh_token_hash))
            .await?
        {
            return Err(AuthenticateError::InvalidToken);
        }

        Ok(auth)
    }

    async fn authenticate_by_password(
        &self,
        txn: &mut Txn,
        user_id: UserId,
        password: UserPassword,
    ) -> Result<(), AuthenticateByPasswordError> {
        let password_hash = self
            .user_repo
            .get_password_hash(txn, user_id)
            .await?
            .ok_or(AuthenticateByPasswordError::InvalidCredentials)?;

        self.password
            .verify(password.into_inner(), password_hash)
            .await
            .map_err(|err| match err {
                PasswordVerifyError::InvalidPassword => {
                    AuthenticateByPasswordError::InvalidCredentials
                }
                PasswordVerifyError::Other(err) => err.into(),
            })
    }

    async fn authenticate_by_refresh_token(
        &self,
        txn: &mut Txn,
        refresh_token: &str,
    ) -> Result<SessionId, AuthenticateByRefreshTokenError> {
        let refresh_token_hash = self.hash.sha256(refresh_token.as_bytes()).into();

        let session = self
            .session_repo
            .get_by_refresh_token_hash(txn, refresh_token_hash)
            .await?
            .ok_or(AuthenticateByRefreshTokenError::Invalid)?;

        let now = self.time.now();
        if now >= session.updated_at + self.config.refresh_token_ttl {
            return Err(AuthenticateByRefreshTokenError::Expired(session.id));
        }

        Ok(session.id)
    }

    fn issue_tokens(&self, user: &User, session_id: SessionId) -> anyhow::Result<Tokens> {
        let refresh_token = self.secret.generate(self.config.refresh_token_length);
        let refresh_token_hash = self.hash.sha256(refresh_token.as_bytes()).into();

        let auth = Authentication {
            user_id: user.id,
            session_id,
            refresh_token_hash,
            admin: user.admin,
            email_verified: user.email_verified,
        };
        let access_token = self
            .jwt
            .sign(Token::from(auth), self.config.access_token_ttl)?;

        Ok(Tokens {
            access_token,
            refresh_token,
            refresh_token_hash,
        })
    }

    async fn invalidate_access_token(
        &self,
        refresh_token_hash: SessionRefreshTokenHash,
    ) -> anyhow::Result<()> {
        self.auth_invalidate_access_token
            .invoke(refresh_token_hash)
            .await
    }

    async fn invalidate_access_tokens(&self, txn: &mut Txn, user_id: UserId) -> anyhow::Result<()> {
        for refresh_token_hash in self
            .session_repo
            .list_refresh_token_hashes_by_user(txn, user_id)
            .await?
        {
            self.auth_invalidate_access_token
                .invoke(refresh_token_hash)
                .await?;
        }

        Ok(())
    }
}

fn access_token_invalidated_key(refresh_token_hash: SessionRefreshTokenHash) -> String {
    format!(
        "access_token_invalidated:{}",
        hex::encode(refresh_token_hash.0)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Token {
    uid: UserId,
    sid: SessionId,
    rt: SessionRefreshTokenHash,
    data: Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Data {
    admin: bool,
    email_verified: bool,
}

impl From<Token> for Authentication {
    fn from(value: Token) -> Self {
        Self {
            user_id: value.uid,
            session_id: value.sid,
            refresh_token_hash: value.rt,
            admin: value.data.admin,
            email_verified: value.data.email_verified,
        }
    }
}

impl From<Authentication> for Token {
    fn from(value: Authentication) -> Self {
        Self {
            uid: value.user_id,
            sid: value.session_id,
            rt: value.refresh_token_hash,
            data: Data {
                admin: value.admin,
                email_verified: value.email_verified,
            },
        }
    }
}
