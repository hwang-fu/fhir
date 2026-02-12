# FHIR R4 Patient Server

A production-ready FHIR R4 server for the **Patient** resource, built with Rust.

Features a PostgreSQL custom extension (PGRX) for storage, full CRUD + search + history, AI-powered features via Claude API, and Docker Compose for one-command deployment.

## Tech Stack

| Layer              | Technology                          |
| ------------------ | ----------------------------------- |
| HTTP Server        | Rust + [Axum](https://github.com/tokio-rs/axum) |
| FHIR Types         | [fhir-sdk](https://crates.io/crates/fhir-sdk) (R4B) |
| Postgres Extension | Rust + [PGRX](https://github.com/pgcentralfoundation/pgrx) |
| LLM Integration    | Claude API (Anthropic)              |
| Containerization   | Docker Compose                      |
| CI/CD              | GitHub Actions                      |

## Architecture

```text
┌──────────┐     ┌────────────────────────────────────────────────┐     ┌───────────────┐
│  Client   │────▶│              server (Axum)                    │────▶│  Claude API    │
│ (curl,    │◀────│  Middleware: Auth → Audit → ReqID → RateLimit │     │ (NL Search,   │
│  Postman) │     │                                                │     │  Chat, Gen)   │
└──────────┘     │  Routes:                                       │     └───────────────┘
                  │   POST   /fhir/Patient          Create         │
                  │   GET    /fhir/Patient/{id}     Read           │
                  │   PUT    /fhir/Patient/{id}     Update         │
                  │   DELETE /fhir/Patient/{id}     Delete         │
                  │   GET    /fhir/Patient?...      Search         │
                  │   GET    /fhir/Patient/{id}/_history  History  │
                  │   POST   /fhir/Patient/$validate Validate     │
                  │   POST   /fhir/Patient/$nl-search AI Search   │
                  │   POST   /fhir/Patient/$generate  AI Generate │
                  │   POST   /fhir/$chat             AI Chat      │
                  │   GET    /metadata               Capability   │
                  │   GET    /health                  Health       │
                  │   GET    /metrics                 Prometheus   │
                  └──────────────────┬───────────────────────────┘
                                     │ SQL (extension functions)
                                     ▼
                  ┌──────────────────────────────────────────────┐
                  │         PostgreSQL + fhir-pg-ext             │
                  │  fhir_put · fhir_get · fhir_update           │
                  │  fhir_delete · fhir_search · fhir_history    │
                  │                                              │
                  │  Tables: fhir_resources, fhir_history        │
                  │  Indexes: GIN (JSONB), BTREE (gender, date)  │
                  └──────────────────────────────────────────────┘
```

## Project Structure

```
fhir/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── core/                     # Shared FHIR types & utilities
│   │   └── src/
│   │       ├── lib.rs            # Re-exports Patient, HumanName, Identifier
│   │       ├── bundle.rs         # FHIR Bundle (searchset, history)
│   │       ├── outcome.rs        # OperationOutcome for errors
│   │       ├── error.rs          # FhirError enum
│   │       └── capability.rs     # CapabilityStatement
│   ├── server/                   # Axum HTTP server
│   │   └── src/
│   │       ├── main.rs           # Entry point, router setup
│   │       ├── config.rs         # Env-var configuration
│   │       ├── routes/           # Endpoint handlers
│   │       ├── middleware/        # Auth, audit, request ID, rate limit, metrics
│   │       ├── db/               # Connection pool & PatientRepository
│   │       ├── ai/               # Claude API client, NL search, generator, chatbot
│   │       └── error.rs          # AppError → OperationOutcome
│   └── pg-ext/                   # PGRX PostgreSQL extension
│       └── src/
│           ├── lib.rs            # Extension entry point
│           ├── storage.rs        # fhir_put, fhir_get, fhir_update, fhir_delete
│           ├── search.rs         # fhir_search with filters & pagination
│           ├── history.rs        # fhir_history, fhir_get_version
│           └── schema.sql        # Table & index definitions
├── docker/
│   ├── postgres/
│   │   ├── Dockerfile            # Multi-stage: build PGRX ext → postgres:17
│   │   └── init.sql              # CREATE EXTENSION on first boot
│   └── server/
│       └── Dockerfile            # Multi-stage: compile server → debian-slim
├── docker-compose.yml
├── examples/                     # Sample data & curl scripts
├── postman/                      # Postman collection
├── Makefile
└── .github/workflows/ci.yml
```

## Quick Start

### Using Docker Compose (recommended)

```bash
# Clone the repository
git clone https://github.com/hwang-fu/fhir.git
cd fhir

# Copy and edit environment variables
cp .env.example .env
# Edit .env to set API_KEY and optionally ANTHROPIC_API_KEY

# Start all services
docker compose up -d --build

# Verify both services are healthy
docker compose ps

# Create a patient
curl -X POST http://localhost:8080/fhir/Patient \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"resourceType": "Patient", "name": [{"family": "Smith", "given": ["John"]}], "gender": "male", "birthDate": "1990-05-15"}'

# Stop services
docker compose down
```

### Local Development (without Docker)

Prerequisites: Rust 1.92+, PostgreSQL 17, [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) 0.16.x

```bash
# Terminal 1 — Start PostgreSQL with the PGRX extension
cargo pgrx run pg17 -p fhir-pg-ext
# In the psql prompt:
#   DROP EXTENSION IF EXISTS fhir_pg_ext CASCADE;
#   CREATE EXTENSION fhir_pg_ext;
# Keep this terminal open.

# Terminal 2 — Start the server
API_KEY="secret123" \
DATABASE_URL="postgres://$USER@localhost:28817/fhir_pg_ext" \
cargo run -p fhir-server

# Terminal 3 — Test
curl http://localhost:8080/health
curl http://localhost:8080/metadata | jq
```

## API Reference

All `/fhir/*` endpoints require the `X-API-Key` header (unless auth is disabled).
Public endpoints (`/metadata`, `/health`, `/metrics`) do not require auth.

### Core CRUD

| Method | Endpoint | Description | Success |
| ------ | -------- | ----------- | ------- |
| `POST` | `/fhir/Patient` | Create patient | `201` + `Location` + `ETag` |
| `GET` | `/fhir/Patient/{id}` | Read patient | `200` + `ETag` + JSON |
| `PUT` | `/fhir/Patient/{id}` | Update patient | `200` + `ETag` |
| `DELETE` | `/fhir/Patient/{id}` | Delete patient (soft) | `204` |

### Search & History

| Method | Endpoint | Description |
| ------ | -------- | ----------- |
| `GET` | `/fhir/Patient?name=&gender=&birthdate=&_count=&_offset=&_sort=` | Search with pagination |
| `GET` | `/fhir/Patient/{id}/_history` | Version history |

**Search parameters:**

| Parameter | Type | Example |
| --------- | ---- | ------- |
| `name` | string (substring) | `name=Smith` |
| `gender` | token | `gender=male` |
| `birthdate` | date with prefix | `birthdate=ge1990-01-01` |
| `_count` | integer | `_count=10` (default 10) |
| `_offset` | integer | `_offset=0` |
| `_sort` | field name | `_sort=-birthdate` (prefix `-` = descending) |

### Extended Features

| Method | Endpoint | Description |
| ------ | -------- | ----------- |
| `POST` | `/fhir/Patient/$validate` | Validate without storing |
| `GET` | `/metadata` | CapabilityStatement |

### AI Features (require `ANTHROPIC_API_KEY`)

| Method | Endpoint | Body | Description |
| ------ | -------- | ---- | ----------- |
| `POST` | `/fhir/Patient/$nl-search` | `{"query": "..."}` | Natural language → FHIR search |
| `POST` | `/fhir/Patient/$generate` | `{"count": 5}` | Generate synthetic patients (max 50) |
| `POST` | `/fhir/$chat` | `{"message": "..."}` | AI chatbot with tool calling |

### Observability

| Method | Endpoint | Description |
| ------ | -------- | ----------- |
| `GET` | `/health` | DB connectivity check (`200`/`503`) |
| `GET` | `/metrics` | Prometheus text format |

## Configuration

All configuration is via environment variables:

| Variable | Required | Default | Description |
| -------- | -------- | ------- | ----------- |
| `DATABASE_URL` | Yes | `host=localhost user=postgres dbname=fhir` | PostgreSQL connection string |
| `BIND_ADDRESS` | No | `0.0.0.0:8080` | Server listen address |
| `API_KEY` | No | _(disabled)_ | API key for `X-API-Key` auth |
| `ANTHROPIC_API_KEY` | No | _(disabled)_ | Enables AI features |
| `CORS_ORIGINS` | No | `*` | Comma-separated allowed origins |
| `RATE_LIMIT_RPS` | No | `100` | Max requests per second |
| `RUST_LOG` | No | `info` | Log level filter |

## Middleware

Requests flow through these layers (outermost first):

1. **Prometheus Metrics** — counts requests, records latency histogram
2. **Tracing** — HTTP-level tracing via `tower-http`
3. **CORS** — configurable origin allowlist
4. **Request ID** — generates or propagates `X-Request-ID`
5. **Audit** — logs POST/PUT/DELETE mutations
6. **Rate Limit** — token-bucket rate limiter (protected routes only)
7. **Auth** — validates `X-API-Key` header (protected routes only)

## Testing

Integration tests use [testcontainers](https://github.com/testcontainers/testcontainers-rs) to spin up a real PostgreSQL instance with the PGRX extension, then exercise all HTTP endpoints through the Axum router.

```bash
# Build the test database image (first time only)
make test-db-image

# Run all tests
make integration-test

# Or directly
cargo test -p fhir-server -- --test-threads=1
```

| Test | What it verifies |
| ---- | ---------------- |
| `test_auth` | Missing / wrong / correct API key |
| `test_crud_lifecycle` | Create → Read → Update → Delete → 404 |
| `test_health` | `GET /health` → 200 healthy |
| `test_history` | Create + update → `/_history` with 2 versions |
| `test_metadata` | `GET /metadata` → CapabilityStatement |
| `test_pagination` | `_count` / `_offset` + pagination links |
| `test_search` | Name, gender, birthdate filters + combined |
| `test_validate` | Valid → 200, invalid → 400 |

## CI/CD

GitHub Actions runs on every push and PR:

| Job | Trigger | Description |
| --- | ------- | ----------- |
| **Format** | push + PR | `cargo fmt --check` |
| **Build** | push + PR | `cargo build` (core + server) |
| **Clippy** | PR only | `cargo clippy -- -D warnings` |
| **Docker** | PR only | `docker compose build` |
| **Integration** | PR only | Build test DB image + `cargo test` |

## License

[MIT](LICENSE)
