use crate::service::error::ServiceError;

/// 用户名 3–32 个字符，只允许 ASCII 字母、数字、下划线（§7）。
///
/// 调用方必须先归一化（trim + lowercase）再校验 —— 否则 `"  ab  "` 会被当成 6 个字符放行。
pub fn validate_username(username: &str) -> Result<(), ServiceError> {
    let len = username.chars().count();

    if len < 3 || len > 32 {
        return Err(ServiceError::InvalidParams(
            "用户名长度需在 3-32 个字符之间".to_owned(),
        ));
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(ServiceError::InvalidParams(
            "用户名只能包含字母、数字和下划线".to_owned(),
        ));
    }

    Ok(())
}

/// 邮箱：恰好一个 `@`，且 `@` 两边都不为空，不含空白字符。
///
/// 故意不用正则：邮箱的完整语法非常宽，写了也只是自欺欺人（踩坑 65/66 还牵扯正则编译和 DoS）。
/// 真正的验证手段是发确认邮件，不是格式校验。
pub fn validate_email(email: &str) -> Result<(), ServiceError> {
    if email.chars().any(char::is_whitespace) {
        return Err(ServiceError::InvalidParams("邮箱不能包含空格".to_owned()));
    }

    let Some((local, domain)) = email.split_once('@') else {
        return Err(ServiceError::InvalidParams("邮箱格式不正确".to_owned()));
    };

    if local.is_empty() || domain.is_empty() || domain.contains('@') {
        return Err(ServiceError::InvalidParams("邮箱格式不正确".to_owned()));
    }

    Ok(())
}

/// 密码至少 8 个**字符**（不是字节）。
///
/// `chars().count()` 才算对：`"密码密码"` 是 4 个字符 / 8 个字节，
/// 用 `len()` 会把它当成 8 位放行（踩坑 63）。
pub fn validate_password(password: &str) -> Result<(), ServiceError> {
    if password.chars().count() < 8 {
        return Err(ServiceError::InvalidParams(
            "密码至少需要 8 个字符".to_owned(),
        ));
    }

    Ok(())
}
