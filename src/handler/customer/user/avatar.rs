use axum::Json;
use axum::extract::multipart::MultipartError;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use serde::Serialize;

use crate::handler::error::ApiError;
use crate::handler::extractor::CustomerUser;
use crate::handler::response::ApiResponse;
use crate::service::common::user::avatar::update_avatar;
use crate::service::error::ServiceError;
use crate::state::AppState;

/// 上传头像的响应：直接给完整 URL，前端拿来就能塞进 <img src>
#[derive(Serialize)]
pub struct AvatarResponse {
    pub avatar: String,
}

/// POST /customer/avatar —— 上传当前用户的头像
///
/// 表单字段名固定为 `file`，单文件。校验和转码都在 service 层。
pub async fn update_avatar_handler(
    CustomerUser { user_id }: CustomerUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<AvatarResponse>>, ApiError> {
    // Field 借用着 multipart，出不了循环，所以只能就地取完字节存到这个变量里
    let mut file_bytes = None;

    while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
        // 先看名字（借用），再取字节（消费 field）—— 顺序反了编译不过
        if field.name() != Some("file") {
            continue;
        }

        file_bytes = Some(field.bytes().await.map_err(multipart_error)?);
        break;
    }；

    let Some(bytes) = file_bytes else {
        return Err(ApiError::Service(ServiceError::InvalidParams(
            "请选择要上传的图片".into(),
        )));
    };

    // user_id 来自 token，不是请求体 —— 前端传什么 id 都不可信
    let key = update_avatar(&state.db, &state.storage, user_id, &bytes).await?;

    // 库里存 key，对外拼成完整 URL（规则只在这里和 UserResponse 两处）
    let avatar = format!("{}/{}", state.storage_public_url, key);

    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        AvatarResponse { avatar },
    )))
}

/// 把 multipart 的解析错误翻成 422
///
/// 这里的错误文本可以安全地给用户看：它说的是「你发的数据格式不对」，
/// 不像 ImageError 那样带解析器内部细节。
fn multipart_error(error: MultipartError) -> ApiError {
    ApiError::Service(ServiceError::InvalidParams(format!(
        "上传数据解析失败：{error}"
    )))
}
