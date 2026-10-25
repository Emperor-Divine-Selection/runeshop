use regex::Regex;
use std::sync::LazyLock;

use crate::service::error::ServiceError;

pub const MIN_PASSWORD_LEN: usize = 8;

pub fn validate(username: &str, email: &str, password: &str) -> Result<(), ServiceError> {
    if username.is_empty() {
        return Err(ServiceError::InvalidParams("用户名不能为空".into()));
    }
    if !is_valid_email(email) {
        return Err(ServiceError::InvalidParams("邮箱格式不正确".into()));
    }
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(ServiceError::InvalidParams(format!(
            "密码至少 {} 位",
            MIN_PASSWORD_LEN
        )));
    }
    Ok(())
}

pub fn is_valid_email(email: &str) -> bool {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[a-zA-Z]{2,}$").unwrap());
    RE.is_match(email)
}
