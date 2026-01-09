# Rustatsu Sync Server

A Rust-based port of the Kotatsu sync server, originally written in Kotlin.
This project serves as a personal deep dive into the Axum framework and the Rust ecosystem, with implementing Clean Architecture principles.

- kotatsu-syncserver: [https://github.com/KotatsuApp/kotatsu-syncserver](https://github.com/KotatsuApp/kotatsu-syncserver)

## Tech Stack

- Language: Rust
- Web Framework: Axum
- Database: PostgreSQL
- Security: Argon2id for password hashing
- Architecture: Clean Architecture (Domain, Usecase, Infrastructure layers)

## Changes

> [!WARNING]
> Incompatible with the original Kotatsu configuration.

1. Database migration: switched to PostgreSQL (Kotatsu uses MySQL/MariadB).
2. Security update: ~~user password are hashed using Argon2id (Kotatsu uses MD5)~~.
3. Performance optimization: refactored the manga saving logic to reduce database overhead.
   Used HashMaps and HashSets to deduplicate data before execution, the system significantly reduces query usage and improves overall query time.

## Usage

The easiest way to run Rustatsu is using Docker. You can pull the image directly from Docker Hub:

```sh
docker pull abduzzy/rustatsu-sync:latest
```

then run with:

```sh
docker run -d \
  --name rustatsu-sync \
  -p 8080:8080 \
  -e DATABASE__HOST=your_db_host \
  -e DATABASE__PORT=5432 \
  -e DATABASE__USERNAME=postgres \
  -e DATABASE__PASSWORD=password \
  -e DATABASE__DATABASE_NAME=rustatsu \
  -e APPLICATION__PORT=8000
  -e APPLICATION__HOST=0.0.0.0
  -e APPLICATION__BASE_URL=http://127.0.0.1:8000
  -e APPLICATION__RUN_MIGRATION=true \
  -e RUST_LOG=info \
  abduzzy/rustatsu-sync:latest
```

### Use Docker Compose

You can use the provided [docker-compose.yaml](https://raw.githubusercontent.com/kido1611/rustatsu-sync-server/refs/heads/main/docker-compose.yaml) file to start using rustatsu.

```yaml docker-compose.yaml
services:
  rustatsu:
    image: abduzzy/rustatsu-sync:latest
    ports:
      - "8000:8000"
    environment:
      - DATABASE__HOST=postgresql
      - DATABASE__PORT=5432
      - DATABASE__USERNAME=postgres
      - DATABASE__PASSWORD=password
      - DATABASE__DATABASE_NAME=rustatsu
      - APPLICATION__PORT=8000
      - APPLICATION__HOST=0.0.0.0
      - APPLICATION__BASE_URL=http://127.0.0.1:8000
      # - APPLICATION__ALLOW_REGISTRATION=true
      # - APPLICATION__RUN_MIGRATION=TRUE
      #
      # - APPLICATION__HMAC_SECRET=7rKPtEepmdUQSLjQuv5RzjI+uCg/GYHdpgbD6t7ZVrM=
      # HMAC secret can be generate using: `openssl rand -base64 32`
      #
      # - JWT__SECRET=my-secret-key
      # - JWT__ISS=rustatsu
      # - JWT__AUD=rustatsu
      - RUST_LOG=info
    restart: always
    depends_on:
      - postgresql
  postgresql:
    image: postgres:17.3-alpine3.20
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
      POSTGRES_DB: rustatsu
    restart: unless-stopped
    volumes:
      - "pgdata:/var/lib/postgresql/data"
volumes:
  pgdata:
```

### Configuration

The following environment variables can be used to configure the app.

| Variable                          | Description                                                                                      |
| --------------------------------- | ------------------------------------------------------------------------------------------------ |
| DATABASE\_\_HOST                  | PostgreSQL host address                                                                          |
| DATABASE\_\_PORT                  | PostgreSQL port (Default: 5432)                                                                  |
| DATABASE\_\_USERNAME              | Database user                                                                                    |
| DATABASE\_\_PASSWORD              | Database password                                                                                |
| DATABASE\_\_DATABASE_NAME         | Name of the database                                                                             |
| APPLICATION\_\_PORT               | Server port (Default: 8080)                                                                      |
| APPLICATION\_\_HOST               | Server host (Default: 0.0.0.0)                                                                   |
| APPLICATION\_\_BASE_URL           | Server public base url. Used by link in e-mail.                                                  |
| APPLICATION\_\_ALLOW_REGISTRATION | Toggle user registration (true/false)                                                            |
| APPLICATION\_\_RUN_MIGRATION      | Automatically run migrations on start (required on first run or after upgrade to setup database) |
| APPLICATION\_\_HMAC_SECRET        | Secret key for hashing (generate with command `openssl rand -base64 32`)                         |
| JWT\_\_SECRET                     | Secret key for JWT signing                                                                       |
| RUST_LOG                          | Logging level (error, warn, info, debug)                                                         |

## License

This project is licensed under the GNU General Public License v3.0.

- Original Project: [kotatsu-syncserver](https://github.com/KotatsuApp/kotatsu-syncserver) (GPL-3.0)
