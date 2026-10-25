pub mod application;
pub mod user;

use crate::model::sea_orm_active_enums::MerchantStatus;

/// 枚举 → 对外字符串；DTO 序列化统一走这里
pub fn status_text(status: &MerchantStatus) -> &'static str {
    match status {
        MerchantStatus::Pending => "pending",
        MerchantStatus::Active => "active",
        MerchantStatus::Frozen => "frozen",
        MerchantStatus::Rejected => "rejected",
    }
}
