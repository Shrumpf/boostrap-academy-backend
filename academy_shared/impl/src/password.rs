use std::sync::Arc;

use academy_di::Build;
use academy_shared_contracts::password::{PasswordService, PasswordVerifyError};
use argon2::{
    password_hash::{self, rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

#[derive(Debug, Clone, Default, Build)]
pub struct PasswordServiceImpl {
    #[state]
    argon2: Arc<Argon2<'static>>,
}

impl PasswordService for PasswordServiceImpl {
    async fn hash(&self, password: String) -> anyhow::Result<String> {
        let argon2 = Arc::clone(&self.argon2);
        let salt = SaltString::generate(&mut OsRng);
        tokio::task::spawn_blocking(move || {
            argon2
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
        })
        .await?
        .map_err(Into::into)
    }

    async fn verify(&self, password: String, hash: String) -> Result<(), PasswordVerifyError> {
        let argon2 = Arc::clone(&self.argon2);
        tokio::task::spawn_blocking(move || {
            let hash =
                PasswordHash::new(&hash).map_err(|err| PasswordVerifyError::Other(err.into()))?;
            argon2
                .verify_password(password.as_bytes(), &hash)
                .map_err(|err| match err {
                    password_hash::Error::Password => PasswordVerifyError::InvalidPassword,
                    err => PasswordVerifyError::Other(err.into()),
                })
        })
        .await
        .map_err(|err| PasswordVerifyError::Other(err.into()))?
    }
}

#[cfg(test)]
mod tests {
    use academy_utils::assert_matches;

    use super::*;

    #[tokio::test]
    async fn hash_verify_ok() {
        // Arrange
        let password = "some user password";

        let sut = PasswordServiceImpl::default();

        // Act
        let hash = sut.hash(password.into()).await.unwrap();
        let result = sut.verify(password.into(), hash).await;

        // Assert
        result.unwrap();
    }

    #[tokio::test]
    async fn hash_verify_invalid_password() {
        // Arrange
        let password = "some user password";
        let password2 = "other password";

        let sut = PasswordServiceImpl::default();

        // Act
        let hash = sut.hash(password.into()).await.unwrap();
        let result = sut.verify(password2.into(), hash).await;

        // Assert
        assert_matches!(result, Err(PasswordVerifyError::InvalidPassword));
    }
}
