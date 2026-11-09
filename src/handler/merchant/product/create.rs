use axum::{Json, extract::State, http::StatusCode};
use sea_orm::prelude::*;

use super::{CreateProductRequest, ProductResponse};

use crate::handler::error::ApiError;
use crate::handler::extractor::MerchantUser;
use crate::handler::response::ApiResponse;
use crate::service::error::ServiceError;
use crate::service::merchant::product::create_product;
use crate::state::AppState;
use crate::store::product::write::{NewItem, NewKind, NewProduct};

/// POST /merchant/products —— 建商品（含种类、物件、价格、库存）
pub async fn create_product_handler(
    MerchantUser { merchant }: MerchantUser,
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ApiResponse<ProductResponse>>, ApiError> {
    let data = build_new_product(req, merchant.id)?;
    let product = create_product(&state.db, merchant.id, data).await?;

    Ok(Json(ApiResponse::ok(
        StatusCode::CREATED.as_u16(),
        ProductResponse::from(product),
    )))
}

/// 把 HTTP 请求体转成 store 层的入参
///
/// `merchant_id` 来自 token，不在请求体里 —— 前端传的任何商户 id 都不可信。
/// 只有 price 的解析会失败，其余都是纯搬运。
fn build_new_product(
    req: CreateProductRequest,
    merchant_id: Uuid,
) -> Result<NewProduct, ServiceError> {
    let mut kinds = Vec::with_capacity(req.kinds.len());

    for kind in req.kinds {
        let mut items = Vec::with_capacity(kind.items.len());

        for item in kind.items {
            let price = item.price.trim().parse::<Decimal>().map_err(|_| {
                ServiceError::InvalidParams(format!("物件「{}」的价格格式不正确", item.name))
            })?;

            items.push(NewItem {
                name: item.name,
                sort_order: item.sort_order,
                price,
                stock: item.stock,
            });
        }

        kinds.push(NewKind {
            name: kind.name,
            sort_order: kind.sort_order,
            items,
        });
    }

    Ok(NewProduct {
        merchant_id,
        name: req.name,
        description: req.description,
        cover_image: req.cover_image,
        kinds,
    })
}
