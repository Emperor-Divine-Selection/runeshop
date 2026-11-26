pub mod application;
pub mod credential;
pub mod customer;
pub mod error;
pub mod merchant;
pub mod operator;
pub mod normalize;
pub mod validate;

/// 登录输入：三端共用（用户名 + 明文密码）。
///
/// 只放「三端完全一样」的东西；角色差异（查哪张表、额外状态检查）
/// 留在各自的 service 模块里，不塞进这个结构。
#[derive(Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}
