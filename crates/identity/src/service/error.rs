use common::code::ApiCode;
use common::password::PasswordError;
use sea_orm::DbErr;
use thiserror::Error;

use crate::service::credential::session::SessionError;
use crate::service::credential::token::TokenError;
use crate::store::error::StoreError;

#[derive(Debug, Error)]
pub enum ServiceError {
    /// 数据库层面的失败（连接、约束、语法……）
    #[error("[Technical] {0}")]
    Database(#[from] DbErr),

    /// 用户发的东西本身有问题：长度、格式、类型
    #[error("[InvalidParams] {0}")]
    InvalidParams(String),

    /// 用户发的东西没问题，但当前状态不接受（重名、撞唯一约束）
    #[error("[Conflict] {0}")]
    Conflict(String),

    #[error("[Technical] {0}")]
    Password(#[from] PasswordError),

    #[error("[Technical] {0}")]
    Token(#[from] TokenError),

    #[error("[Technical] {0}")]
    Cache(#[from] SessionError),

    /// 凭证错误：登录时用户名不存在 **或** 密码不匹配。
    /// 两条分支必须共用这一个变体，否则等于送给攻击者一个用户枚举接口（踩坑 51）。
    #[error("[InvalidCredentials] {0}")]
    InvalidCredentials(String),
}

impl ServiceError {
    /// 变体 → 业务码（§6）。这是唯一的映射点。
    ///
    /// 四个技术变体都映射到 `Internal`：对外只需要说「内部错误」，
    /// 具体是数据库、缓存还是密码子系统，留在日志里（Display 的 `[Technical]` 前缀）。
    pub fn code(&self) -> ApiCode {
        match self {
            Self::Database(_) => ApiCode::Internal,
            Self::InvalidParams(_) => ApiCode::InvalidParams,
            Self::Conflict(_) => ApiCode::Conflict,
            Self::Password(_) => ApiCode::Internal,
            Self::Token(_) => ApiCode::Internal,
            Self::Cache(_) => ApiCode::Internal,
            Self::InvalidCredentials(_) => ApiCode::InvalidCredentials,
        }
    }
}

/// 展平：剥掉 store 的壳，让 service 只看到 `DbErr`（踩坑 46）。
impl From<StoreError> for ServiceError {
    fn from(value: StoreError) -> Self {
        match value {
            StoreError::Technical(e) => Self::Database(e),
        }
    }
}
