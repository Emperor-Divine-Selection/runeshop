CREATE TYPE operator_status AS ENUM ('active', 'disabled');

CREATE TABLE operator_roles (
    id         uuid PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    permission varchar(32) NOT NULL,      -- super_admin / reviewer
    name       varchar(64) NOT NULL
);

CREATE UNIQUE INDEX uq_operator_roles_permission ON operator_roles (permission);

-- 关联表：没有独立生命周期，复合主键即可
CREATE TABLE operator_role_permissions (
    role_id    uuid NOT NULL REFERENCES operator_roles(id) ON DELETE CASCADE,
    permission varchar(64) NOT NULL,      -- merchant_application.review ...
    PRIMARY KEY (role_id, permission)
);

CREATE TABLE operators (
    id            uuid PRIMARY KEY,
    role_id       uuid NOT NULL REFERENCES operator_roles(id),
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    username      varchar(32)  NOT NULL,
    password_hash varchar(255),           -- NULL = 未激活
    email         varchar(255) NOT NULL,
    status        operator_status NOT NULL DEFAULT 'active'
);

CREATE UNIQUE INDEX uq_operators_username ON operators (username);
CREATE UNIQUE INDEX uq_operators_email ON operators (email);

-- 角色由平台固定，种子写进迁移
INSERT INTO operator_roles (id, permission, name) VALUES
    ('01a2137f-4c44-7c03-9e2e-e9ce4b0e7b44', 'super_admin', '超级管理员'),
    ('01a2137f-4c44-7f56-9a53-484241c909da', 'reviewer', '审核员');

-- super_admin 不授权限（代码按 permission 特判放行）；
-- reviewer 先给审核相关的两个权限码
INSERT INTO operator_role_permissions (role_id, permission)
SELECT id, 'merchant_application.list'
  FROM operator_roles WHERE permission = 'reviewer';

INSERT INTO operator_role_permissions (role_id, permission)
SELECT id, 'merchant_application.review'
  FROM operator_roles WHERE permission = 'reviewer';
