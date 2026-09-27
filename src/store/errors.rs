use sea_orm::DbErr;
use std::fmt::{self};

/// 错误类型
#[derive(Debug)]
pub enum StoreError {
    NotFound(String),      // 查不到
    Business(String),      // 业务规则不通过
    InvalidParams(String), // 参数本身不合法
    Conflict(String),      // 撞唯一约束（用户名/邮箱重复）
    Technical(DbErr),      // DB 挂了、SQL 语法错
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "[NotFound] {msg}"),
            Self::Business(msg) => write!(f, "[Business] {msg}"),
            Self::InvalidParams(msg) => write!(f, "[InvalidParams] {msg}"),
            Self::Conflict(msg) => write!(f, "[Conflict] {msg}"),
            Self::Technical(e) => write!(f, "[Technical] {e}"),
        }
    }
}

impl From<DbErr> for StoreError {
    fn from(value: DbErr) -> Self {
        Self::Technical(value)
    }
}
