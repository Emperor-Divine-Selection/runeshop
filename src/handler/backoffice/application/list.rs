use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use crate::handler::backoffice::application::{ApplicationResponse, parse_status};
use crate::handler::error::ApiError;
use crate::handler::extractor::BackofficeUser;
use crate::handler::response::{ApiResponse, PageResponse};
use crate::service::backoffice::application::{ListApplicationsQuery, list_applications};
use crate::state::AppState;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;

/// 列表查询参数：全部可选，缺省由这里补默认值
#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// GET /backoffice/applications —— 申请单列表（可按状态筛选、分页）
pub async fn list_applications_handler(
    _: BackofficeUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<PageResponse<ApplicationResponse>>>, ApiError> {
    let status = match query.status.as_deref() {
        Some(raw) => Some(parse_status(raw)?),
        None => None,
    };

    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let page_size = query.page_size.unwrap_or(DEFAULT_PAGE_SIZE);

    let (rows, total) = list_applications(
        &state.db,
        ListApplicationsQuery {
            status,
            keyword: query.keyword,
            page,
            page_size,
        },
    )
    .await?;

    let items = rows.into_iter().map(ApplicationResponse::from).collect();

    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        PageResponse {
            items,
            total,
            page,
            page_size,
        },
    )))
}
