pub fn normalize_username(username: &str) -> String {
    username.trim().to_lowercase().to_string()
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase().to_string()
}
