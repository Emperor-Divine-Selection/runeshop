use sea_orm::{ConnectionTrait, sqlx::types::Uuid};

use crate::{
    model::merchant_applications::Model,
    service::error::ServiceError,
    store::merchant_application::find_merchant_application_by_user_id,
};

/// 查当前用户提交的商户申请；没提交过返 `None`
///
/// 不区分状态 —— 前端拿 `status` 自己决定显示「表单 / 审核中 / 已通过 / 已驳回」
pub async fn find_own_application<C: ConnectionTrait>(
    db: &C,
    user_id: Uuid,
) -> Result<Option<Model>, ServiceError> {
    Ok(find_merchant_application_by_user_id(db, user_id).await?)
}
