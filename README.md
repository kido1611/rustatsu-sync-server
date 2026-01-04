# Rustatsu Sync Server

A Rust-based port of the Kotatsu sync server, originally written in Kotlin.
This project serves as a personal deep dive into the Axum framework and the Rust ecosystem, with implementing Clean Architecture principles.

## Tech Stack

- Language: Rust
- Web Framework: Axum
- Database: PostgreSQL
- Security: Argon2id for password hashing
- Architecture: Clean Architecture (Domain, Usecase, Infrastructure layers)

## Changes

> [!WARNING]
> This version is incompatible with the original Kotatsu configuration.

1. Database migration: switched to PostgreSQL (Kotatsu uses MySQL/MariadB).
2. Security update: ~~user password are hashed using Argon2id (Kotatsu uses MD5)~~.
3. Performance optimization: refactored the manga saving logic to reduce database overhead.
   Used HashMaps and HashSets to deduplicate data before execution, the system significantly reduces query usage and improves overall query time.

