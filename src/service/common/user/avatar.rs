use crate::service::error::ServiceError;
use crate::storage::object::put_object;
use crate::store::user::update_user_avatar;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use object_store::ObjectStore;
use sea_orm::ConnectionTrait;
use sea_orm::sqlx::types::Uuid;
use std::sync::Arc;

const MAX_AVATAR_BYTES: usize = 5 * 1024 * 1024;
const AVATAR_EDGE: u32 = 512;
const JPEG_QUALITY: u8 = 90;

pub async fn update_avatar<C: ConnectionTrait>(
    db: &C,
    storage: &Arc<dyn ObjectStore>,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<String, ServiceError> {
    if bytes.len() > MAX_AVATAR_BYTES {
        return Err(ServiceError::InvalidParams(format!(
            "图片不能超过 {} MB",
            MAX_AVATAR_BYTES / 1024 / 1024
        )));
    }

    let decoded = image::load_from_memory(bytes)?;

    let avatar = decoded.resize_to_fill(AVATAR_EDGE, AVATAR_EDGE, FilterType::Lanczos3);

    let mut jpeg = Vec::new();
    {
        let encoder = JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY);
        avatar.write_with_encoder(encoder)?;
    }

    let key = format!("{user_id}/{}.jpg", Uuid::new_v4());
    put_object(storage, &key, jpeg).await?;
    update_user_avatar(db, &user_id, &key).await?;
    Ok(key)
}
