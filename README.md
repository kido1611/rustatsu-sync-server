# Rustatsu-Sync

Porting of kotatsu-sync server from Kotlin to Rust. This project is used for personal project to learning Rust language and Axum framework.

## Changes

Incompatible with kotatsu configuration.

1. This project using PostgreSQL (kotatsu using MySQL/MariadB).
2. User password is hashed using Argon2id (kotatsu using md5).
3. Some tweak before save manga to reduce query usage (Collected with HashMap / HashSet to reduce duplicate).
