-- 状态用 PG 原生 enum：类型本身就拒绝非法值，不靠 CHECK 兜底
CREATE TYPE application_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE merchant_applications (
    id          uuid PRIMARY KEY,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    shop_name   varchar(64)  NOT NULL,
    username    varchar(32)  NOT NULL,
    email       varchar(255) NOT NULL,
    status      application_status NOT NULL DEFAULT 'pending',
    reviewed_by uuid,
    reviewed_at timestamptz,
    review_note text,

    CONSTRAINT chk_review_consistency CHECK (
        (status = 'pending'  AND reviewed_by IS NULL     AND reviewed_at IS NULL)
     OR (status <> 'pending' AND reviewed_by IS NOT NULL AND reviewed_at IS NOT NULL)
    )
);

-- 同一个商户名 / 邮箱，同时只能有一条待审申请
CREATE UNIQUE INDEX uq_pending_username
    ON merchant_applications (username) WHERE status = 'pending';

CREATE UNIQUE INDEX uq_pending_email
    ON merchant_applications (email) WHERE status = 'pending';
