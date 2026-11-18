#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ApiCode {
    Ok = 0,
    InvalidCredentials = 1001,
    Unauthorized = 1002,
    InvalidParams = 1003,
    Conflict = 1004,
    Internal = 5000,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_spec() {
        assert_eq!(ApiCode::Ok as i32, 0);
        assert_eq!(ApiCode::InvalidCredentials as i32, 1001);
        assert_eq!(ApiCode::Unauthorized as i32, 1002);
        assert_eq!(ApiCode::InvalidParams as i32, 1003);
        assert_eq!(ApiCode::Conflict as i32, 1004);
        assert_eq!(ApiCode::Internal as i32, 5000);
    }
}
