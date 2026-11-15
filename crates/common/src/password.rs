use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("[Technical] {0}")]
    Hash(#[from] argon2::password_hash::Error),
}

pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())?
        .to_string())
}

pub fn verify_password(stored_hash: &str, plain: &str) -> Result<bool, PasswordError> {
    match Argon2::default().verify_password(plain.as_bytes(), stored_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::PasswordInvalid) => Ok(false),
        Err(e) => Err(PasswordError::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_has_phc_prefix() {
        let hash = hash_password("pass1234").expect("哈希应该成功");
        assert!(hash.starts_with("$argon2id$"), "实际: {hash}");
    }

    #[test]
    fn correct_password_is_true() {
        let hash = hash_password("pass1234").expect("哈希应该成功");
        assert!(verify_password(&hash, "pass1234").expect("校验应该成功"));
    }

    #[test]
    fn wrong_pass() {
        let hash = hash_password("pass1234").expect("哈希应该成功");
        assert!(!verify_password(&hash, "wrong").expect("校验应该成功"));
    }

    #[test]
    fn corrupt_hash_is_error_not_false() {
        let result = verify_password("这不是一个 PHC 字符串", "pass1234");
        assert!(result.is_err(), "坏哈希应该是 Err，实际: {result:?}");
    }
}
