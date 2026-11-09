use sea_orm::ConnectionTrait;

use crate::service::common::validation::validate;
use crate::service::error::ServiceError;
use crate::store::backoffice_user::{
    NewBackofficeUser, create_backoffice_user, find_backoffice_user_by_username,
};

/// 确保存在引导超管：按用户名查，已存在就跳过，不存在就创建。
pub async fn ensure_bootstrap_admin<C: ConnectionTrait>(
    db: &C,
    username: &str,
    email: &str,
    password: &str,
) -> Result<(), ServiceError> {
    let username = username.trim().to_lowercase();
    let email = email.trim().to_lowercase();

    if find_backoffice_user_by_username(db, &username)
        .await?
        .is_some()
    {
        tracing::info!(%username, "引导超管已存在，跳过创建");
        return Ok(());
    }

    validate(&username, &email, password)?;

    create_backoffice_user(
        db,
        NewBackofficeUser {
            username: username.clone(),
            password: password.to_string(),
            email,
        },
    )
    .await?;

    tracing::info!(%username, "已创建引导超管");
    Ok(())
}
