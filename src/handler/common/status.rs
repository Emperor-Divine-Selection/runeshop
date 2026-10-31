use crate::model::sea_orm_active_enums::{ApplicationStatus, MerchantStatus};

/// 商户经营状态 → 对外字符串（active / frozen）
pub fn merchant_status_text(status: &MerchantStatus) -> &'static str {
    match status {
        MerchantStatus::Active => "active",
        MerchantStatus::Frozen => "frozen",
    }
}

/// 申请单审核状态 → 对外字符串（pending / approved / rejected）
pub fn application_status_text(status: &ApplicationStatus) -> &'static str {
    match status {
        ApplicationStatus::Pending => "pending",
        ApplicationStatus::Approved => "approved",
        ApplicationStatus::Rejected => "rejected",
    }
}
