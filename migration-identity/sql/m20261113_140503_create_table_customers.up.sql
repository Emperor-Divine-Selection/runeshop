CREATE TABLE customers (
    id            uuid PRIMARY KEY,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    username      varchar(32)  NOT NULL UNIQUE,
    password_hash varchar(255) NOT NULL,
    email         varchar(255) NOT NULL UNIQUE
);
