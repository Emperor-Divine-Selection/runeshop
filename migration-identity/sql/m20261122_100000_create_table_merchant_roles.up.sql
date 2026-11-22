CREATE TABLE merchant_roles (
    id         uuid PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    permission varchar(32) NOT NULL,
    name       varchar(64) NOT NULL
);

CREATE UNIQUE INDEX uq_merchant_roles_permission ON merchant_roles (permission);

-- 关联表：没有独立生命周期，复合主键即可，不需要 id / 时间戳
CREATE TABLE merchant_role_permissions (
    role_id    uuid NOT NULL REFERENCES merchant_roles(id) ON DELETE CASCADE,
    permission varchar(64) NOT NULL,
    PRIMARY KEY (role_id, permission)
);

-- 角色是平台固定的，不是用户创建的：种子数据写进迁移，保证每个环境一致。
-- owner 的权限在代码里按 permission 特判放行，所以这里不预置权限；
-- staff 的权限等商品 / 订单接口落地后再加，避免现在瞎猜权限码。
INSERT INTO merchant_roles (id, permission, name) VALUES
    ('01a203c0-dc25-73e6-b3e8-b5e9dcaf8032', 'owner', '店主'),
    ('01a203c0-dc25-766a-ac3d-893a73e387b7', 'staff', '店小二');
