use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::model::operator_role_permissions::Column;
use crate::model::prelude::OperatorRolePermissions;
use crate::store::error::StoreError;

pub async fn find_permissions<C: ConnectionTrait>(
    db: &C,
    role_id: Uuid,
) -> Result<Vec<String>, StoreError> {
    let rows = OperatorRolePermissions::find()
        .filter(Column::RoleId.eq(role_id))
        .all(db)
        .await?;

    Ok(rows.into_iter().map(|row| row.permission).collect())
}
