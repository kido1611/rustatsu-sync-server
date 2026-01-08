-- Add down migration script here
DROP INDEX IF EXISTS users_password_reset_token_hash_unique;

ALTER TABLE
    users DROP COLUMN IF EXISTS password_reset_token_hash,
    DROP COLUMN IF EXISTS password_reset_token_expires_at;
