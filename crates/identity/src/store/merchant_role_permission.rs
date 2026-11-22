use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::model::merchant_role_permissions::Column;
use crate::model::prelude::MerchantRolePermissions;
use crate::store::error::StoreError;

/// 某个角色被授予的所有权限码。
pub async fn find_permissions<C: ConnectionTrait>(
    db: &C,
    role_id: Uuid,
) -> Result<Vec<String>, StoreError> {
    let rows = MerchantRolePermissions::find()
        .filter(Column::RoleId.eq(role_id))
        .all(db)
        .await?;

    Ok(rows.into_iter().map(|row| row.permission).collect())
}
