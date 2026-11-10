//! 密码哈希：**纯计算，不碰数据库**。
//!
//! 放在 store 层是因为「哈希格式怎么存」属于存储细节，
//! 而且 service 和 store 都要用 —— 放 service 会让 store 反向依赖 service。
//!
//! 三个调用方（customer / operator / merchant）共用这一份，
//! 以前是三份逐字节拷贝，改 argon2 参数要同时改三处。

use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use sea_orm::DbErr;

use crate::store::error::StoreError;

/// 把明文密码哈希成 `password_hash`
pub fn hash_password(plain: &str) -> Result<String, StoreError> {
    Ok(Argon2::default()
        .hash_password(plain.as_bytes())
        .map_err(|e| StoreError::Technical(DbErr::Custom(e.to_string())))?
        .to_string())
}

/// 校验明文密码是否匹配已存的哈希
pub fn verify_password(stored_hash: &str, plain: &str) -> bool {
    Argon2::default()
        .verify_password(plain.as_bytes(), stored_hash)
        .is_ok()
}
