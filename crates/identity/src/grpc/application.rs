use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::model::merchant_applications::Model;
use crate::model::sea_orm_active_enums::ApplicationStatus as DomainStatus;
use crate::service::application::{SubmitInput, approve, list, reject, submit};

use super::{
    ApproveMerchantApplicationRequest, ApproveMerchantApplicationResponse, IdentityState,
    ListMerchantApplicationsRequest, ListMerchantApplicationsResponse,
    RejectMerchantApplicationRequest, RejectMerchantApplicationResponse,
    MerchantApplication, SubmitMerchantApplicationRequest, SubmitMerchantApplicationResponse,
    ok_status, to_business_status, to_transport_status,
};

pub async fn submit_merchant_application(
    grpc: &IdentityState,
    request: Request<SubmitMerchantApplicationRequest>,
) -> Result<Response<SubmitMerchantApplicationResponse>, Status> {
    let input = request.into_inner();

    let result = submit(
        &grpc.db,
        SubmitInput {
            shop_name: input.shop_name,
            username: input.username,
            email: input.email,
        },
    )
    .await;

    match result {
        Ok(id) => Ok(Response::new(SubmitMerchantApplicationResponse {
            status: Some(ok_status()),
            id: id.to_string(),
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(SubmitMerchantApplicationResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}

pub async fn list_merchant_applications(
    grpc: &IdentityState,
    request: Request<ListMerchantApplicationsRequest>,
) -> Result<Response<ListMerchantApplicationsResponse>, Status> {
    let input = request.into_inner();
    let status = parse_status_filter(input.status)?;

    let result = list(&grpc.db, status, input.page, input.page_size).await;

    match result {
        Ok(page) => Ok(Response::new(ListMerchantApplicationsResponse {
            status: Some(ok_status()),
            applications: page.items.into_iter().map(to_proto_application).collect(),
            total: page.total,
            page: page.page,
            page_size: page.page_size,
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(ListMerchantApplicationsResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}

/// UNSPECIFIED = 全部；非法值走传输层 400。
fn parse_status_filter(value: i32) -> Result<Option<DomainStatus>, Status> {
    match super::ApplicationStatus::try_from(value) {
        Ok(super::ApplicationStatus::Unspecified) => Ok(None),
        Ok(super::ApplicationStatus::Pending) => Ok(Some(DomainStatus::Pending)),
        Ok(super::ApplicationStatus::Approved) => Ok(Some(DomainStatus::Approved)),
        Ok(super::ApplicationStatus::Rejected) => Ok(Some(DomainStatus::Rejected)),
        Err(_) => Err(Status::invalid_argument("status 不合法")),
    }
}

fn to_proto_status(status: DomainStatus) -> i32 {
    match status {
        DomainStatus::Pending => super::ApplicationStatus::Pending as i32,
        DomainStatus::Approved => super::ApplicationStatus::Approved as i32,
        DomainStatus::Rejected => super::ApplicationStatus::Rejected as i32,
    }
}

fn to_proto_application(model: Model) -> MerchantApplication {
    MerchantApplication {
        id: model.id.to_string(),
        shop_name: model.shop_name,
        username: model.username,
        email: model.email,
        status: to_proto_status(model.status),
        review_note: model.review_note.unwrap_or_default(),
        created_at: model.created_at.to_rfc3339(),
        reviewed_at: model
            .reviewed_at
            .map(|t| t.to_rfc3339())
            .unwrap_or_default(),
    }
}

pub async fn approve_merchant_application(
    grpc: &IdentityState,
    request: Request<ApproveMerchantApplicationRequest>,
) -> Result<Response<ApproveMerchantApplicationResponse>, Status> {
    let input = request.into_inner();

    let application_id = Uuid::parse_str(&input.application_id)
        .map_err(|_| Status::invalid_argument("application_id 不是合法 UUID"))?;
    let reviewer_id = Uuid::parse_str(&input.reviewer_id)
        .map_err(|_| Status::invalid_argument("reviewer_id 不是合法 UUID"))?;

    let result = approve(&grpc.db, application_id, reviewer_id).await;

    match result {
        Ok(merchant_id) => Ok(Response::new(ApproveMerchantApplicationResponse {
            status: Some(ok_status()),
            merchant_id: merchant_id.to_string(),
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(ApproveMerchantApplicationResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}

pub async fn reject_merchant_application(
    grpc: &IdentityState,
    request: Request<RejectMerchantApplicationRequest>,
) -> Result<Response<RejectMerchantApplicationResponse>, Status> {
    let input = request.into_inner();

    let application_id = Uuid::parse_str(&input.application_id)
        .map_err(|_| Status::invalid_argument("application_id 不是合法 UUID"))?;
    let reviewer_id = Uuid::parse_str(&input.reviewer_id)
        .map_err(|_| Status::invalid_argument("reviewer_id 不是合法 UUID"))?;

    let result = reject(&grpc.db, application_id, reviewer_id, input.note).await;

    match result {
        Ok(()) => Ok(Response::new(RejectMerchantApplicationResponse {
            status: Some(ok_status()),
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(RejectMerchantApplicationResponse {
                status: Some(status),
            })),
            None => Err(to_transport_status(err)),
        },
    }
}
