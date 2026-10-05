use fred::prelude::*;
use std::time::Duration;

use crate::cache::errors::CacheError;

/// 写入一个带过期时间的值。ttl 为 None 时永不过期
pub async fn set_ex<V>(
    client: &Client,
    key: &str,
    value: V,
    ttl: Option<Duration>,
) -> Result<(), CacheError>
where
    V: TryInto<Value> + Send,
    <V as TryInto<Value>>::Error: Into<Error> + Send,
{
    // Duration → fred 的过期枚举。EX 收秒，as_secs() 正好
    let expire = ttl.map(|d| Expiration::EX(d.as_secs() as i64));

    // set 的 5 个参数一个都不能省（fred 没默认参数）：
    // key / value / expire / options(NX|XX，用不到) / get(写完是否回读 → false)
    // 返回类型标成 ()：不关心 SET 回了什么，只关心有没有报错
    let _: () = client
        .set(key, value, expire, None, false)
        .await
        .map_err(CacheError::from)?;

    Ok(())
}

/// 读一个值。键不存在或已过期都返回 `Ok(None)`
///
/// `R` 由调用方决定（`Option<i32>` / `Option<String>`），fred 靠它决定怎么解析字节
pub async fn get<R>(client: &Client, key: &str) -> Result<Option<R>, CacheError>
where
    R: FromValue,
{
    // 这个 `let v: Option<R>` 的标注是必须的 —— 少了它编译器不知道按什么类型解析
    let v: Option<R> = client.get(key).await.map_err(CacheError::from)?;
    Ok(v)
}

/// 删一个键。键不存在不算错误
pub async fn del(client: &Client, key: &str) -> Result<(), CacheError> {
    // del 的参数名在 fred 源码里叫 `keys`（复数，能一次删多个），传单个也行
    let _: () = client.del(key).await.map_err(CacheError::from)?;
    Ok(())
}
