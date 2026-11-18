use serde::Serialize;

use crate::code::ApiCode;

#[derive(Debug, Default, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub meta: Meta,
    pub trace_id: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 成功：200 + code:0 + data
    pub fn ok(data: T) -> Self {
        Self {
            code: ApiCode::Ok as i32,
            message: "ok".to_owned(),
            data: Some(data),
            meta: Meta::default(),
            trace_id: String::new(), // 步 7 补，先空着
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_envelope_has_code_zero_and_data() {
        let resp = ApiResponse::ok("hello".to_owned());
        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains(r#""code":0"#), "实际: {json}");
        assert!(json.contains(r#""data":"hello""#), "实际: {json}");
    }

    #[test]
    fn meta_empty_serializes_to_empty_object() {
        let json = serde_json::to_string(&Meta::default()).unwrap();
        assert_eq!(json, "{}", "没有分页时 meta 必须是 {{}}，实际: {json}");
    }

    #[test]
    fn meta_with_pagination_keeps_fields() {
        let meta = Meta {
            total: Some(128),
            page: Some(1),
            page_size: Some(20),
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains(r#""total":128"#), "实际: {json}");
    }
}
