CREATE TYPE shop_status AS ENUM ('active','frozen');

CREATE TABLE shops (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    name varchar(64) NOT NULL,
    status shop_status NOT NULL DEFAULT 'active'
);

CREATE UNIQUE INDEX uq_shops_name ON shops (lower(name));
