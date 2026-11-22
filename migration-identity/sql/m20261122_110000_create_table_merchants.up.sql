CREATE TYPE merchant_status AS ENUM ('active', 'frozen');

CREATE TABLE merchants (
    id            uuid PRIMARY KEY,
    shop_id       uuid NOT NULL REFERENCES shops(id),
    role_id       uuid NOT NULL REFERENCES merchant_roles(id),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    username      varchar(32)  NOT NULL,
    -- NULL = 还没设密码（未激活），等邀请链接走完再填
    password_hash varchar(255),
    email         varchar(255) NOT NULL,
    status        merchant_status NOT NULL DEFAULT 'active'
);

CREATE UNIQUE INDEX uq_merchants_username ON merchants (username);
CREATE UNIQUE INDEX uq_merchants_email ON merchants (email);

-- 常用查询：一家店的所有账号（店主 + 店小二）
CREATE INDEX idx_merchants_shop_id ON merchants (shop_id);
