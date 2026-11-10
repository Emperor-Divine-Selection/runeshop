pub mod avatar;

use sea_orm::{ConnectionTrait, sqlx::types::Uuid};

use crate::model::identity::customers::Model;
use crate::service::error::ServiceError;
use crate::store::user::find_by_id;

/// 取指定用户资料；查不到返 `NotFound`
pub async fn profile<C: ConnectionTrait>(db: &C, user_id: &Uuid) -> Result<Model, ServiceError> {
    let Some(user) = find_by_id(db, user_id).await? else {
        return Err(ServiceError::NotFound("用户不存在".into()));
    };
    Ok(user)
}
