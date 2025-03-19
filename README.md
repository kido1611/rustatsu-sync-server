# RUSTATSU-Sync

Porting of kotatsu-sync server from Kotlin to Rust. This project is used for personal project and learning Rust language and Axum framework.

## Changes

Incompatible with kotatsu configuration.

1. This project using PostgreSQL (kotatsu using MySQL/MariadB).
2. User password hashing using Argon2id (kotatsu using md5).
