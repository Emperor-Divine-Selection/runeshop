use fred::error::Error;
use sea_orm::DbErr;
use std::fmt;

use crate::cache::errors::CacheError;
use crate::store::errors::StoreError;

/// 业务错误类型
#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    Business(String),
    InvalidParams(String),
    Conflict(String),
    Database(DbErr),
    Cache(Error),
}

impl From<StoreError> for ServiceError {
    fn from(value: StoreError) -> Self {
        match value {
            StoreError::NotFound(msg) => Self::NotFound(msg),
            StoreError::Technical(e) => Self::Database(e),
        }
    }
}

impl From<CacheError> for ServiceError {
    fn from(value: CacheError) -> Self {
        match value {
            CacheError::Technical(e) => Self::Cache(e),
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
            Self::Database(e) => write!(f, "[Technical] {}", e),
            Self::Cache(e) => write!(f, "[Technical] {}", e),
        }
    }
}
