use crate::model::users::*;
use sea_orm::DeleteResult;
use sea_orm::Set;
use sea_orm::prelude::*;

pub struct UserStore {
    db: DatabaseConnection,
}

impl UserStore {
    /// 传入一个连接的引用，内部存一份 clone。
    /// DatabaseConnection 内部是 Arc（引用计数），clone 几乎零成本，
    /// 所以这样持有不带生命周期，以后放进共享状态（AppState）毫无障碍。
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    /// 创建用户。
    /// 三个 Set 的字段进 SQL，其余字段（id/created_at/updated_at/xp...）
    /// 未设置 → INSERT 语句不含这些列 → 数据库默认值生效
    pub async fn add_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<Model, DbErr> {
        // 第一步：在内存里摆好一行待写入的数据
        let user = ActiveModel {
            username: Set(username.to_owned()),
            email: Set(email.to_owned()),
            password_hash: Set(password_hash.to_owned()),
            // 其余字段：未设置状态，数据库 default 生效
            ..Default::default()
        };

        // 第二步：交给连接插入，返回带 id/时间戳的完整 Model
        user.insert(&self.db).await
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<DeleteResult, DbErr> {
        Entity::delete_by_id(id).exec(&self.db).await
    }

    pub async fn delete_by_username(&self, username: &str) -> Result<DeleteResult, DbErr> {
        Entity::delete_many()
            .filter(Column::Username.eq(username))
            .exec(&self.db)
            .await
    }

    pub async fn delete_by_email(&self, email: &str) -> Result<DeleteResult, DbErr> {
        Entity::delete_many()
            .filter(Column::Email.eq(email))
            .exec(&self.db)
            .await
    }

    /// 改用户名。用户不存在时返回 RecordNotFound。
    pub async fn update_username_by_id(&self, id: i32, new_username: &str) -> Result<Model, DbErr> {
        // 1. 先查出来 —— .one() 给 Option<Model>，查不到要当场处理
        //    ok_or_else 把 None 变成 Err（RecordNotFound），再加 ? 传出去
        let user = Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("user id={id} 不存在")))?;

        // 2. Model → ActiveModel（现在才轮到 into()）
        let mut user: ActiveModel = user.into();

        // 3. 改字段 —— 只有被 Set 的字段会进 UPDATE 语句
        user.username = Set(new_username.to_owned());

        // 4. 写回，返回更新后的 Model
        user.update(&self.db).await
    }

    pub async fn update_password_hash_by_id(
        &self,
        id: i32,
        new_password_hash: &str,
    ) -> Result<Model, DbErr> {
        let user = Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("user id={id} 不存在")))?;

        let mut user: ActiveModel = user.into();

        user.password_hash = Set(new_password_hash.to_owned());

        user.update(&self.db).await
    }

    pub async fn update_email_by_id(&self, id: i32, new_email: &str) -> Result<Model, DbErr> {
        let user = Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("user id={id} 不存在")))?;

        let mut user: ActiveModel = user.into();

        user.email = Set(new_email.to_owned());

        user.update(&self.db).await
    }

    pub async fn update_avatar_url_by_id(
        &self,
        id: i32,
        new_avatar_url: &str,
    ) -> Result<Model, DbErr> {
        let user = Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("user id={id} 不存在")))?;

        let mut user: ActiveModel = user.into();

        user.avatar_url = Set(Some(new_avatar_url.to_owned()));

        user.update(&self.db).await
    }

    pub async fn update_bio_by_id(&self, id: i32, new_bio: Option<String>) -> Result<Model, DbErr> {
        let user = Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("user id={id} 不存在")))?;

        let mut user: ActiveModel = user.into();

        user.bio = Set(new_bio);

        user.update(&self.db).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(&self.db).await
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::Username.eq(name))
            .one(&self.db)
            .await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::Email.eq(email))
            .one(&self.db)
            .await
    }
}
