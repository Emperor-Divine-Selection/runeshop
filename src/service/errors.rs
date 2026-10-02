use sea_orm::DbErr;
use std::fmt;

use crate::store::errors::StoreError;

/// 业务错误类型
#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    Business(String),
    InvalidParams(String),
    Conflict(String),
    Technical(DbErr),
}

impl From<StoreError> for ServiceError {
    fn from(value: StoreError) -> Self {
        match value {
            StoreError::NotFound(msg) => Self::NotFound(msg),
            StoreError::Technical(e) => Self::Technical(e),
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "[NotFound] {msg}"),
            Self::Business(msg) => write!(f, "[Business] {msg}"),
            Self::InvalidParams(msg) => write!(f, "[InvalidParams] {msg}"),
            Self::Conflict(msg) => write!(f, "[Conflict] {msg}"),
            Self::Technical(e) => write!(f, "[Technical] {}", e),
        }
    }
}
