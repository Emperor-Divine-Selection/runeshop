use common::password::hash_password;
use sea_orm::ConnectionTrait;
use uuid::Uuid;

use crate::model::customers::Model;
use crate::service::error::ServiceError;
use crate::service::normalize::{normalize_email, normalize_username};
use crate::service::validate::{validate_email, validate_password, validate_username};
use crate::store::customer::{NewCustomer, create, find_by_email, find_by_username};

#[derive(Debug)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// 注册。**不自动登录** —— 不建 session，调用方拿到的只有新行（§7）。
pub async fn register<C: ConnectionTrait>(
    db: &C,
    input: RegisterInput,
) -> Result<Model, ServiceError> {
    // 1. 先归一化：后面的校验和查重都必须基于归一化后的值
    let username = normalize_username(&input.username);
    let email = normalize_email(&input.email);

    // 2. 再校验：参数不合法就不该产生数据库往返
    validate_username(&username)?;
    validate_email(&email)?;
    validate_password(&input.password)?;

    // 3. 查重（便宜）。用归一化后的值查，否则 "ALICE" 查不到库里的 "alice"
    if find_by_username(db, &username).await?.is_some() {
        return Err(ServiceError::Conflict("用户名已被占用".to_owned()));
    }

    if find_by_email(db, &email).await?.is_some() {
        return Err(ServiceError::Conflict("邮箱已被注册".to_owned()));
    }

    // 4. 最后才哈希（贵，一次约 50ms）。放前面等于让人用重复请求白嫖 CPU
    let password_hash = hash_password(&input.password)?;

    // 5. 建号。id 由应用生成（UUID v7），数据库没有默认值
    let new_customer = NewCustomer {
        id: Uuid::now_v7(),
        username,
        email,
        password_hash,
    };

    Ok(create(db, new_customer).await?)
}
