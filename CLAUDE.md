# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

### Development and Testing
- `cargo test` - Run all tests in workspace
- `cargo test -p jp-postal-code-core` - Run tests for specific crate
- `cargo check` - Quick compile check
- `cargo clippy` - Linter
- `cargo fmt` - Format code

### Docker Development
- `docker compose build` - Build application image
- `docker compose up -d` - Start services (PostgreSQL + app on port 8000)
- `docker compose down` - Stop services

## Architecture Overview

This is a Rust workspace with 3 main crates implementing a Japanese postal code lookup service:

### Crate Structure
- **jp-postal-code-core**: Core domain models and normalization logic for postal data
- **jp-postal-code-util**: Utilities for downloading and parsing postal code ZIP files from Japan Post
- **jp-postal-code**: Main HTTP service with Axum web framework, PostgreSQL repository, and business logic

### Key Components
- **Repository Pattern**: `UtfKenAllRepository` trait with PostgreSQL and ephemeral implementations in `jp-postal-code/src/infra/`
- **Use Cases**: Business logic in `jp-postal-code/src/usecase.rs` for searching and updating postal data
- **Data Source**: Downloads `ken_all_utf8.zip` from Japan Post website automatically
- **Normalization**: Complex town name normalization logic in `jp-postal-code-core/src/normalize/`

### Testing
- Uses `insta` for snapshot testing (many `.snap` files in project)
- Run `cargo insta review` to review snapshot changes
- PostgreSQL tests require a running database
- Ephemeral tests run in-memory for faster execution

### Database
- PostgreSQL with SQLx for migrations and queries
- Migration files in `migrations/` directory
- Auto-initializes postal data on first startup if database is empty

### API Endpoints
- `GET /api/search?postal_code=XXX&page_size=N&page_token=TOKEN` - Search postal codes
- `POST /api/update` - Download latest postal data from Japan Post

### Environment
- Set `DATABASE_URL` for PostgreSQL connection
- Uses `.env` file if present
- Default HTTP server listens on `HTTP_SERVER_ADDR` (0.0.0.0:80 in Docker)
