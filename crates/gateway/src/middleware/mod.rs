pub mod auth;

use common::role::Role;

/// 中间件验完 token 后放到 request extensions 里的身份。
/// handler 通过 `Extension<CurrentUser>` 取，不需要再碰 Cookie。
#[derive(Clone)]
pub struct CurrentUser {
    pub id: String,
    pub role: Role,
}
