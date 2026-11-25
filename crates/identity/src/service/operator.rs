use std::env;

use common::password::{hash_password, verify_password};
use common::role::Role;
use sea_orm::ConnectionTrait;
use uuid::Uuid;

use crate::model::sea_orm_active_enums::OperatorStatus;
use crate::service::LoginInput;
use crate::service::credential::{session::SessionStore, token::generate_token};
use crate::service::error::ServiceError;
use crate::service::normalize::{normalize_email, normalize_username};
use crate::service::validate::{validate_email, validate_password, validate_username};
use crate::store::operator::{NewOperator, create, find_any, find_by_id, find_by_username};
use crate::store::{operator_role, operator_role_permission};

const BOOTSTRAP_USERNAME: &str = "BOOTSTRAP_ADMIN_USERNAME";
const BOOTSTRAP_EMAIL: &str = "BOOTSTRAP_ADMIN_EMAIL";
const BOOTSTRAP_PASSWORD: &str = "BOOTSTRAP_ADMIN_PASSWORD";

/// 运营登录。和商户登录同构：查表 → 验密 → 查状态 → 写 session。
pub async fn login<C: ConnectionTrait>(
    db: &C,
    session: &SessionStore,
    input: LoginInput,
) -> Result<String, ServiceError> {
    let username = normalize_username(&input.username);

    let Some(operator) = find_by_username(db, &username).await? else {
        return Err(ServiceError::InvalidCredentials(
            "用户名或密码错误".to_owned(),
        ));
    };

    // 未激活（还没设密码）与密码错返回同一条错误，防账号枚举
    let Some(password_hash) = operator.password_hash.as_deref() else {
        return Err(ServiceError::InvalidCredentials(
            "用户名或密码错误".to_owned(),
        ));
    };

    if !verify_password(password_hash, &input.password)? {
        return Err(ServiceError::InvalidCredentials(
            "用户名或密码错误".to_owned(),
        ));
    }

    // 密码验证通过后才检查状态
    if operator.status != OperatorStatus::Active {
        return Err(ServiceError::Unauthorized("账号已被停用".to_owned()));
    }

    let token = generate_token()?;
    session.save(&token, Role::Operator, &operator.id).await?;

    Ok(token)
}

/// 运营资料：账号 + 权限（`/operator/me` 用这个）。
#[derive(Debug)]
pub struct OperatorProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub permissions: Vec<String>,
}

pub async fn me_operator<C: ConnectionTrait>(
    db: &C,
    operator_id: Uuid,
) -> Result<OperatorProfile, ServiceError> {
    let Some(operator) = find_by_id(db, operator_id).await? else {
        return Err(ServiceError::Unauthorized("用户不存在".to_owned()));
    };

    if operator.status != OperatorStatus::Active {
        return Err(ServiceError::Unauthorized("账号已被停用".to_owned()));
    }

    let Some(role) = operator_role::find_by_id(db, operator.role_id).await? else {
        tracing::error!(
            operator_id = %operator_id,
            role_id = %operator.role_id,
            "运营的角色不存在"
        );
        return Err(ServiceError::Internal("运营数据不完整".to_owned()));
    };

    // super_admin 返回 "*"（全部权限）
    let permissions = if role.permission == "super_admin" {
        vec!["*".to_owned()]
    } else {
        operator_role_permission::find_permissions(db, operator.role_id).await?
    };

    Ok(OperatorProfile {
        id: operator.id,
        username: operator.username,
        email: operator.email,
        permissions,
    })
}

/// 启动时建第一个超管（幂等）。
///
/// - `operators` 非空 → 跳过
/// - 没配 `BOOTSTRAP_ADMIN_*` → 记一条 warn 后跳过（不阻断启动）
pub async fn bootstrap_super_admin<C: ConnectionTrait>(db: &C) -> Result<(), ServiceError> {
    if find_any(db).await?.is_some() {
        tracing::info!("operators 非空，跳过引导超管");
        return Ok(());
    }

    let (Ok(username), Ok(email), Ok(password)) = (
        env::var(BOOTSTRAP_USERNAME),
        env::var(BOOTSTRAP_EMAIL),
        env::var(BOOTSTRAP_PASSWORD),
    ) else {
        tracing::warn!("未配置 BOOTSTRAP_ADMIN_*，跳过引导超管");
        return Ok(());
    };

    let username = normalize_username(&username);
    let email = normalize_email(&email);

    validate_username(&username)?;
    validate_email(&email)?;
    validate_password(&password)?;

    let Some(role) = operator_role::find_by_permission(db, "super_admin").await? else {
        tracing::error!("operator_roles 缺少 super_admin");
        return Err(ServiceError::Internal("缺少 super_admin 角色".to_owned()));
    };

    let password_hash = hash_password(&password)?;

    let operator = create(
        db,
        NewOperator {
            id: Uuid::now_v7(),
            role_id: role.id,
            username,
            password_hash,
            email,
        },
    )
    .await?;

    tracing::info!(
        operator_id = %operator.id,
        username = %operator.username,
        "已创建引导超管"
    );

    Ok(())
}
