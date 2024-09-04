use std::future::Future;

use academy_models::{
    oauth2::{OAuth2Link, OAuth2LinkId},
    user::UserId,
};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait OAuth2Repository<Txn: Send + Sync + 'static>: Send + Sync + 'static {
    fn list_links_by_user(
        &self,
        txn: &mut Txn,
        user_id: UserId,
    ) -> impl Future<Output = anyhow::Result<Vec<OAuth2Link>>> + Send;

    fn get_link(
        &self,
        txn: &mut Txn,
        link_id: OAuth2LinkId,
    ) -> impl Future<Output = anyhow::Result<Option<OAuth2Link>>> + Send;

    fn create_link(
        &self,
        txn: &mut Txn,
        oauth2_link: &OAuth2Link,
    ) -> impl Future<Output = Result<(), OAuth2RepoError>> + Send;

    fn delete_link(
        &self,
        txn: &mut Txn,
        link_id: OAuth2LinkId,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;
}

#[derive(Debug, Error)]
pub enum OAuth2RepoError {
    #[error("An oauth2 link already exists for this provider and remote user id.")]
    Conflict,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "mock")]
impl<Txn: Send + Sync + 'static> MockOAuth2Repository<Txn> {
    pub fn with_list_links_by_user(mut self, user_id: UserId, result: Vec<OAuth2Link>) -> Self {
        self.expect_list_links_by_user()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(user_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_get_link(mut self, link_id: OAuth2LinkId, result: Option<OAuth2Link>) -> Self {
        self.expect_get_link()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(link_id),
            )
            .return_once(|_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_create(mut self, link: OAuth2Link, result: Result<(), OAuth2RepoError>) -> Self {
        self.expect_create_link()
            .once()
            .with(mockall::predicate::always(), mockall::predicate::eq(link))
            .return_once(|_, _| Box::pin(std::future::ready(result)));
        self
    }

    pub fn with_delete_link(mut self, link_id: OAuth2LinkId, result: bool) -> Self {
        self.expect_delete_link()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(link_id),
            )
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
