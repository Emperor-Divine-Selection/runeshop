use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("[Technical] {0}")]
    Technical(#[from] DbErr),
}
