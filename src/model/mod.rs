//! entity 层：**按数据库 schema 分目录**，一一对应。
//!
//! ```text
//! src/model/identity/   ← identity schema（customers / operators）
//! src/model/merchant/   ← merchant schema（merchants / merchant_applications）
//! src/model/catalog/    ← catalog schema（products + 4 张子表）
//! ```
//!
//! ⚠️ 这些文件由 `sea-orm-cli generate entity -s <schema>` 生成，
//! 但**必须分三次生成再手工合并**（每个 schema 都产出自己的 `mod.rs` / `prelude.rs` /
//! `sea_orm_active_enums.rs`，同名文件会撞）。**本文件是手写的，别让生成覆盖。**

pub mod catalog;
pub mod identity;
pub mod merchant;
