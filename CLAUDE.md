# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

### Development and Testing
- `cargo test` - Run all tests in workspace
- `cargo test -p jp-postal-code-core` - Run tests for specific crate
- `cargo check` - Quick compile check
- `cargo clippy` - Linter
- `cargo fmt` - Format code
- `cargo insta review` - Review snapshot test changes

### Just Task Runner Commands
- `just` - List all available tasks
- `just check` - Run clippy and protobuf lint
- `just test` - Run all tests including doc tests
- `just fmt` - Format Rust code and protobuf files
- `just gen-proto` - Generate Rust code from protobuf definitions
- `just dev` - Start development server
- `just migrate` - Run database migrations
- `just setup-tools-mac` - Install required tools on macOS

### Docker Development
- `docker compose build` - Build application image
- `docker compose up -d` - Start services (PostgreSQL + HTTP on port 8000 + gRPC on port 50051)
- `docker compose down` - Stop services

## Architecture Overview

This is a Rust workspace with 5 crates implementing a Japanese postal code lookup service:

### Crate Structure
- **jp-postal-code-core**: Core domain models and normalization logic for postal data
- **jp-postal-code-util**: Utilities for downloading and parsing postal code ZIP files from Japan Post
- **jp-postal-code**: Main HTTP/gRPC service with Axum web framework, Tonic gRPC, PostgreSQL repository, and business logic
- **jp-postal-code-proto**: Generated protobuf code for gRPC service definitions
- **jp-postal-code-update-database**: CLI tool for updating postal code database

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

#### HTTP API (port 8000)
- `GET /api/search?postal_code=XXX&page_size=N&page_token=TOKEN` - Search postal codes

#### gRPC API (port 50051)
- `jp_postal_code.v1.PostalAddressService/SearchPostalAddress` - Search postal codes via gRPC
- Server reflection is enabled for service discovery

### Environment
- Set `DATABASE_URL` for PostgreSQL connection
- Uses `.env` file if present
- `HTTP_SERVER_ADDR` - HTTP server address (default: 0.0.0.0:80 in Docker, 127.0.0.1:8000 in dev)
- `GRPC_SERVER_ADDR` - gRPC server address (default: 0.0.0.0:50051)

### Protocol Buffers
- Proto definitions in `proto/jp_postal_code/v1/`
- Generated Rust code in `jp-postal-code-proto/src/_gen/`
- Uses `buf` for linting, formatting, and code generation
