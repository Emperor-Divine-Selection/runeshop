use std::collections::HashSet;

use sea_orm::sqlx::types::Decimal;
use sea_orm::{ConnectionTrait, TransactionTrait, sqlx::types::Uuid};

use crate::model::catalog::products::Model;
use crate::service::error::ServiceError;
use crate::store::product::read::find_product_by_merchant_and_name;
use crate::store::product::write::{NewProduct, create_product as store_create_product};

const MAX_PRODUCT_NAME_LEN: usize = 120;
const MAX_KIND_NAME_LEN: usize = 50;
const MAX_ITEM_NAME_LEN: usize = 50;

/// 建商品（含种类、物件、价格、库存）
pub async fn create_product<T>(
    db: &T,
    merchant_id: Uuid,
    mut req: NewProduct,
) -> Result<Model, ServiceError>
where
    T: ConnectionTrait + TransactionTrait,
{
    // 归一化必须在校验和查重之前：否则「红色」和「 红色」会同时通过，
    // 然后在数据库撞唯一约束变成 500
    normalize(&mut req);
    validate(&req)?;

    // 同一商户下不能有同名商品：唯一约束在库里，这里是为了给出 409 而不是 500
    if find_product_by_merchant_and_name(db, &merchant_id, &req.name)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict(format!(
            "你已经有一个叫「{}」的商品了",
            req.name
        )));
    }

    Ok(store_create_product(db, req).await?)
}

/// 去掉首尾空白；描述归一化成空串时转 None
fn normalize(req: &mut NewProduct) {
    req.name = req.name.trim().to_string();
    req.description = req
        .description
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string);

    for kind in &mut req.kinds {
        kind.name = kind.name.trim().to_string();
        for item in &mut kind.items {
            item.name = item.name.trim().to_string();
        }
    }
}

/// 校验商品树：格式问题返 422，请求体内重名返 409
///
/// 这里的「重名」只可能出现在**同一个请求体内部**（种类和物件都是新造的，
/// 不会和历史数据冲突），所以不需要查库，用 HashSet 在内存里判就行。
fn validate(req: &NewProduct) -> Result<(), ServiceError> {
    if req.name.is_empty() {
        return Err(ServiceError::InvalidParams("商品名称不能为空".into()));
    }
    if req.name.chars().count() > MAX_PRODUCT_NAME_LEN {
        return Err(ServiceError::InvalidParams(format!(
            "商品名称不能超过 {MAX_PRODUCT_NAME_LEN} 个字"
        )));
    }
    if req.kinds.is_empty() {
        return Err(ServiceError::InvalidParams(
            "至少要有一个种类，否则这个商品永远卖不出去".into(),
        ));
    }

    let mut seen_kinds: HashSet<&str> = HashSet::new();

    for kind in &req.kinds {
        if kind.name.is_empty() {
            return Err(ServiceError::InvalidParams("种类名称不能为空".into()));
        }
        if kind.name.chars().count() > MAX_KIND_NAME_LEN {
            return Err(ServiceError::InvalidParams(format!(
                "种类名称不能超过 {MAX_KIND_NAME_LEN} 个字"
            )));
        }
        if !seen_kinds.insert(kind.name.as_str()) {
            return Err(ServiceError::Conflict(format!(
                "种类「{}」写了两遍",
                kind.name
            )));
        }
        if kind.items.is_empty() {
            return Err(ServiceError::InvalidParams(format!(
                "种类「{}」下面至少要有一个物件",
                kind.name
            )));
        }

        let mut seen_items: HashSet<&str> = HashSet::new();

        for item in &kind.items {
            if item.name.is_empty() {
                return Err(ServiceError::InvalidParams("物件名称不能为空".into()));
            }
            if item.name.chars().count() > MAX_ITEM_NAME_LEN {
                return Err(ServiceError::InvalidParams(format!(
                    "物件名称不能超过 {MAX_ITEM_NAME_LEN} 个字"
                )));
            }
            if !seen_items.insert(item.name.as_str()) {
                return Err(ServiceError::Conflict(format!(
                    "种类「{}」下的物件「{}」写了两遍",
                    kind.name, item.name
                )));
            }
            if item.price < Decimal::ZERO {
                return Err(ServiceError::InvalidParams(format!(
                    "物件「{}」的价格不能为负",
                    item.name
                )));
            }
            if item.stock < 0 {
                return Err(ServiceError::InvalidParams(format!(
                    "物件「{}」的库存不能为负",
                    item.name
                )));
            }
        }
    }

    Ok(())
}
