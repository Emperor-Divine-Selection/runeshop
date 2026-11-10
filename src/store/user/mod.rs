use sea_orm::ActiveValue::Set;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::model::identity::customers::Entity as Customers;
use crate::model::identity::customers::{ActiveModel, Column, Model};
use crate::store::error::StoreError;
use crate::store::password::hash_password;

/// 创建顾客的入参；`password` 是明文，由 store 内部哈希成 `password_hash`
pub struct NewCustomer {
    pub username: String,
    pub password: String,
    pub email: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

/// 创建顾客；不查重（查重是 service 层的业务规则）
pub async fn create_user<C: ConnectionTrait>(
    db: &C,
    data: NewCustomer,
) -> Result<Model, StoreError> {
    let username = data.username.trim().to_string();
    let password = data.password.trim().to_string();
    let email = data.email.trim().to_string();

    Ok(ActiveModel {
        username: Set(username),
        password_hash: Set(hash_password(&password)?),
        email: Set(email),
        avatar: Set(data.avatar),
        bio: Set(data.bio),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

/// 更新顾客头像
pub async fn update_user_avatar<C: ConnectionTrait>(
    db: &C,
    user_id: &Uuid,
    avatar: &str,
) -> Result<Model, StoreError> {
    let Some(user) = find_by_id(db, user_id).await? else {
        return Err(StoreError::NotFound("用户不存在".into()));
    };

    let mut active: ActiveModel = user.into();
    active.avatar = Set(Some(avatar.to_string()));

    Ok(active.update(db).await?)
}

/// 按用户名查顾客，查不到返回 Ok(None)
pub async fn find_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Customers::find()
        .filter(Column::Username.eq(username))
        .one(db)
        .await?)
}

/// 按 id 查顾客，查不到返回 Ok(None)
pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    user_id: &Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(Customers::find_by_id(*user_id).one(db).await?)
}

/// 按邮箱查顾客，查不到返回 Ok(None)
pub async fn find_by_email<C: ConnectionTrait>(
    db: &C,
    email: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Customers::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await?)
}
