use academy_models::{
    auth::Login,
    session::{DeviceName, Session, SessionId},
    user::UserId,
};
use serde::{Deserialize, Serialize};

use super::user::ApiUser;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiSession {
    pub id: SessionId,
    pub user_id: UserId,
    pub device_name: Option<DeviceName>,
    pub last_update: i64,
}

impl From<Session> for ApiSession {
    fn from(value: Session) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            device_name: value.device_name,
            last_update: value.updated_at.timestamp(),
        }
    }
}

#[derive(Serialize)]
pub struct ApiLogin {
    user: ApiUser,
    session: ApiSession,
    access_token: String,
    refresh_token: String,
}

impl From<Login> for ApiLogin {
    fn from(value: Login) -> Self {
        Self {
            user: value.user_composite.into(),
            session: value.session.into(),
            access_token: value.access_token,
            refresh_token: value.refresh_token,
        }
    }
}
