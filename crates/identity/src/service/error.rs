use common::code::ApiCode;
use common::password::PasswordError;
use sea_orm::DbErr;
use thiserror::Error;

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
}

impl ServiceError {
    /// 变体 → 业务码（§6）。这是唯一的映射点。
    ///
    /// 三个技术变体都映射到 `Internal`：对外只需要说「内部错误」，
    /// 具体是数据库还是缓存，留在日志里（Display 的 `[Technical]` 前缀）。
    pub fn code(&self) -> ApiCode {
        match self {
            Self::Database(_) => ApiCode::Internal,
            Self::InvalidParams(_) => ApiCode::InvalidParams,
            Self::Conflict(_) => ApiCode::Conflict,
            Self::Password(_) => ApiCode::Internal,
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
