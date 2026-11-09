use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::TransactionSession;
use sea_orm::TransactionTrait;
use sea_orm::sqlx::types::{Decimal, Uuid};

use crate::model::product_inventory;
use crate::model::product_items;
use crate::model::product_kinds;
use crate::model::product_prices;
use crate::model::products;
use crate::model::products::Model;
use crate::store::error::StoreError;

#[derive(Debug)]
pub struct NewProduct {
    pub merchant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub kinds: Vec<NewKind>,
}

#[derive(Debug)]
pub struct NewKind {
    pub name: String,
    pub sort_order: i32,
    pub items: Vec<NewItem>,
}

#[derive(Debug)]
pub struct NewItem {
    pub name: String,
    pub sort_order: i32,
    pub price: Decimal,
    pub stock: i32,
}

pub async fn create_product<T: TransactionTrait>(
    db: &T,
    data: NewProduct,
) -> Result<Model, StoreError> {
    let txn = db.begin().await?;

    let product = products::ActiveModel {
        merchant_id: Set(data.merchant_id),
        name: Set(data.name),
        description: Set(data.description),
        cover_image: Set(data.cover_image),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    for kind in data.kinds {
        let kind_row = product_kinds::ActiveModel {
            product_id: Set(product.id),
            name: Set(kind.name),
            sort_order: Set(kind.sort_order),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        for item in kind.items {
            let item_row = product_items::ActiveModel {
                kind_id: Set(kind_row.id),
                name: Set(item.name),
                sort_order: Set(item.sort_order),
                ..Default::default()
            }
            .insert(&txn)
            .await?;

            product_prices::ActiveModel {
                item_id: Set(item_row.id),
                amount: Set(item.price),
                ..Default::default()
            }
            .insert(&txn)
            .await?;

            product_inventory::ActiveModel {
                item_id: Set(item_row.id),
                stock: Set(item.stock),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }
    }

    txn.commit().await?;
    Ok(product)
}
