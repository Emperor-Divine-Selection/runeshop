use sea_orm::ActiveValue::Set;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::model::identity::operators::{ActiveModel, Column, Model};
use crate::model::identity::operators::Entity as Operators;
use crate::store::error::StoreError;
use crate::store::password::hash_password;

pub struct NewBackofficeUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

pub async fn create_backoffice_user<C: ConnectionTrait>(
    db: &C,
    data: NewBackofficeUser,
) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        username: Set(data.username),
        password_hash: Set(hash_password(&data.password)?),
        email: Set(data.email),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn find_backoffice_user_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Operators::find()
        .filter(Column::Username.eq(username))
        .one(db)
        .await?)
}

pub async fn find_backoffice_user_by_id<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(Operators::find_by_id(id).one(db).await?)
}

pub async fn find_backoffice_user_by_email<C: ConnectionTrait>(
    db: &C,
    email: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Operators::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await?)
}

/// 批量按 id 查运营，用于「申请列表」批量补审核人名字
///
/// 跨域没有外键、不能 JOIN，所以只能是「先查列表 → 收集 id → 再查一次」
pub async fn find_backoffice_users_by_ids<C: ConnectionTrait>(
    db: &C,
    ids: Vec<Uuid>,
) -> Result<Vec<Model>, StoreError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Operators::find()
        .filter(Column::Id.is_in(ids))
        .all(db)
        .await?)
}
