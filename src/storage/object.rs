use std::sync::Arc;

use object_store::{ObjectStore, ObjectStoreExt, aws::AmazonS3Builder, path::Path};

use crate::{config::Config, storage::error::StorageError};

/// 按配置建一个指向 Garage 的 S3 客户端
///
/// 返回 `Arc<dyn ObjectStore>` 而不是具体的 `AmazonS3`：
/// 一是 `ObjectStore` 是 trait，具体类型会漏到 AppState 上；
/// 二是将来换成别的实现（本地磁盘、云厂商）只改这个函数，调用方不动。
pub fn build_store(config: &Config) -> Result<Arc<dyn ObjectStore>, StorageError> {
    let store = AmazonS3Builder::new()
        .with_endpoint(&config.storage_endpoint)
        .with_region(&config.storage_region)
        .with_bucket_name(&config.storage_bucket)
        .with_access_key_id(&config.storage_access_key)
        .with_secret_access_key(&config.storage_secret_key)
        // 本地 Garage 是 http，SDK 默认只允许 https，不显式放开会被拒
        .with_allow_http(true)
        // Garage 只支持 path-style：bucket 在路径里，不拼成子域名
        .with_virtual_hosted_style_request(false)
        .build()?;

    Ok(Arc::new(store))
}

/// 往 bucket 里写一个对象
///
/// `key` 是桶内相对路径（如 `<user_id>/<uuid>.jpg`），**不含桶名** ——
/// 桶名已经配在客户端里了，再拼一次会变成桶里的 `avatars/` 目录。
pub async fn put_object(
    store: &Arc<dyn ObjectStore>,
    key: &str,
    bytes: Vec<u8>,
) -> Result<(), StorageError> {
    store.put(&Path::from(key), bytes.into()).await?;
    Ok(())
}
