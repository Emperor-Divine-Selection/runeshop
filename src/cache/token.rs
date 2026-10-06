use fred::prelude::*;
use std::time::Duration;

use crate::cache::errors::CacheError;
use crate::cache::kv;

/// token 存活时长：24小时。service 传了具体值就用它
const DEFAULT_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// key 前缀。将来还有 user:1:xxx 等，靠前缀区分避免撞车
const KEY_PREFIX: &str = "token:";

/// 拼出 Valkey 里的完整 key
fn key(token: &str) -> String {
    format!("{KEY_PREFIX}{token}")
}

/// 生成 32 字节随机数 -> hex 编码成 64 字符
pub fn generate() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    hex::encode(bytes)
}

///  存 token → user_id；`ttl` 传 `None` 时用默认 24 小时
pub async fn save(
    client: &Client,
    token: &str,
    user_id: i32,
    ttl: Option<Duration>,
) -> Result<(), CacheError> {
    kv::set_ex(
        client,
        &key(token),
        user_id,
        Some(ttl.unwrap_or(DEFAULT_TTL)),
    )
    .await
}

/// 取 token 对应的 user_id；不存在或已过期都返 `Ok(None)`
pub async fn get(client: &Client, token: &str) -> Result<Option<i32>, CacheError> {
    kv::get(client, &key(token)).await
}

/// 注销：删掉 token
pub async fn delete(client: &Client, token: &str) -> Result<(), CacheError> {
    kv::del(client, &key(token)).await
}
