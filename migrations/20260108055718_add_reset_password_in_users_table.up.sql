-- Add up migration script here
ALTER TABLE
    users
ADD
    COLUMN password_reset_token_hash CHAR(64),
ADD
    COLUMN password_reset_token_expires_at BIGINT;

CREATE UNIQUE INDEX users_password_reset_token_hash_unique ON users(password_reset_token_hash);
