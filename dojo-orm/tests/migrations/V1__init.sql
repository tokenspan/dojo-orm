CREATE TYPE user_role AS ENUM ('admin', 'user');

CREATE TABLE users
(
    id         uuid PRIMARY KEY,
    name       TEXT      NOT NULL,
    email      TEXT      NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);