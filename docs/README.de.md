[English](../README.md) | **Deutsch** | [中文](README.zh.md)

# FHIR R4 Patient Server

Ein produktionsreifer FHIR-R4-Server für die **Patient**-Ressource, gebaut mit Rust.

Mit einer PostgreSQL Custom Extension (PGRX) als Speicher, vollem CRUD + Suche + History, KI-Features via Claude API und Docker Compose für Ein-Befehl-Deployment.

## Tech Stack

| Schicht            | Technologie                         |
| ------------------ | ----------------------------------- |
| HTTP Server        | Rust + [Axum](https://github.com/tokio-rs/axum) |
| FHIR Typen         | [fhir-sdk](https://crates.io/crates/fhir-sdk) (R4B) |
| Postgres Extension | Rust + [PGRX](https://github.com/pgcentralfoundation/pgrx) |
| LLM Integration    | Claude API (Anthropic)              |
| Containerisierung  | Docker Compose                      |
| CI/CD              | GitHub Actions                      |

## Architektur

<p align="center">
  <img src="../diagrams/system-architecture.svg" alt="Systemarchitektur" width="800"/>
</p>

### Datenbankschema

<p align="center">
  <img src="../diagrams/db-schema.drawio.svg" alt="Datenbankschema" width="700"/>
</p>

## Projektstruktur

```
fhir/
├── Cargo.toml                    # Workspace-Root
├── crates/
│   ├── core/                     # Gemeinsame FHIR Typen & Hilfsfunktionen
│   │   └── src/
│   │       ├── lib.rs            # Re-Exports: Patient, HumanName, Identifier
│   │       ├── bundle.rs         # FHIR Bundle (searchset, history)
│   │       ├── outcome.rs        # OperationOutcome für Fehler
│   │       ├── error.rs          # FhirError Enum
│   │       └── capability.rs     # CapabilityStatement
│   ├── server/                   # Axum HTTP Server
│   │   └── src/
│   │       ├── main.rs           # Einstiegspunkt, Router-Setup
│   │       ├── config.rs         # Konfiguration via Umgebungsvariablen
│   │       ├── routes/           # Endpoint-Handler
│   │       ├── middleware/        # Auth, Audit, Request-ID, Rate-Limit, Metriken
│   │       ├── db/               # Verbindungspool & PatientRepository
│   │       ├── ai/               # Claude API Client, NL-Suche, Generator, Chatbot
│   │       └── error.rs          # AppError → OperationOutcome
│   └── pg-ext/                   # PGRX PostgreSQL Extension
│       └── src/
│           ├── lib.rs            # Extension-Einstiegspunkt
│           ├── storage.rs        # fhir_put, fhir_get, fhir_update, fhir_delete
│           ├── search.rs         # fhir_search mit Filtern & Pagination
│           ├── history.rs        # fhir_history, fhir_get_version
│           └── schema.sql        # Tabellen- & Index-Definitionen
├── docker/
│   ├── postgres/
│   │   ├── Dockerfile            # Multi-Stage: PGRX Ext bauen → postgres:17
│   │   └── init.sql              # CREATE EXTENSION beim ersten Start
│   └── server/
│       └── Dockerfile            # Multi-Stage: Server kompilieren → debian-slim
├── docker-compose.yml
├── examples/                     # Beispieldaten & curl-Skripte
├── postman/                      # Postman Collection
├── Makefile
└── .github/workflows/ci.yml
```

## Schnellstart

### Mit Docker Compose (empfohlen)

```bash
# Repository klonen
git clone https://github.com/hwang-fu/fhir.git
cd fhir

# Umgebungsvariablen kopieren und bearbeiten
cp .env.example .env
# .env bearbeiten: API_KEY setzen und optional ANTHROPIC_API_KEY

# Alle Services starten
docker compose up -d --build

# Prüfen ob beide Services laufen
docker compose ps

# Einen Patienten anlegen
curl -X POST http://localhost:8080/fhir/Patient \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"resourceType": "Patient", "name": [{"family": "Smith", "given": ["John"]}], "gender": "male", "birthDate": "1990-05-15"}'

# Services stoppen
docker compose down
```

### Lokale Entwicklung (ohne Docker)

Voraussetzungen: Rust 1.92+, PostgreSQL 17, [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) 0.16.x

```bash
# Terminal 1 — PostgreSQL mit PGRX Extension starten
cargo pgrx run pg17 -p fhir-pg-ext
# Im psql-Prompt:
#   DROP EXTENSION IF EXISTS fhir_pg_ext CASCADE;
#   CREATE EXTENSION fhir_pg_ext;
# Dieses Terminal offen lassen.

# Terminal 2 — Server starten
API_KEY="secret123" \
DATABASE_URL="postgres://$USER@localhost:28817/fhir_pg_ext" \
cargo run -p fhir-server

# Terminal 3 — Testen
curl http://localhost:8080/health
curl http://localhost:8080/metadata | jq
```

## API-Referenz

Alle `/fhir/*`-Endpunkte brauchen den `X-API-Key`-Header (es sei denn Auth ist deaktiviert).
Öffentliche Endpunkte (`/metadata`, `/health`, `/metrics`) brauchen keine Authentifizierung.

### CRUD-Operationen

| Methode | Endpunkt | Beschreibung | Erfolg |
| ------- | -------- | ------------ | ------ |
| `POST` | `/fhir/Patient` | Patient anlegen | `201` + `Location` + `ETag` |
| `GET` | `/fhir/Patient/{id}` | Patient lesen | `200` + `ETag` + JSON |
| `PUT` | `/fhir/Patient/{id}` | Patient aktualisieren | `200` + `ETag` |
| `DELETE` | `/fhir/Patient/{id}` | Patient löschen (soft) | `204` |

<details>
<summary>CRUD Request-Flow Diagramm</summary>

<p align="center">
  <img src="../diagrams/crud-req-flow.drawio.svg" alt="CRUD Request Flow" width="800"/>
</p>
</details>

### Suche & History

| Methode | Endpunkt | Beschreibung |
| ------- | -------- | ------------ |
| `GET` | `/fhir/Patient?name=&gender=&birthdate=&_count=&_offset=&_sort=` | Suche mit Pagination |
| `GET` | `/fhir/Patient/{id}/_history` | Versionshistorie |

**Suchparameter:**

| Parameter | Typ | Beispiel |
| --------- | --- | -------- |
| `name` | String (Teilstring) | `name=Smith` |
| `gender` | Token | `gender=male` |
| `birthdate` | Datum mit Präfix | `birthdate=ge1990-01-01` |
| `_count` | Integer | `_count=10` (Standard 10) |
| `_offset` | Integer | `_offset=0` |
| `_sort` | Feldname | `_sort=-birthdate` (Präfix `-` = absteigend) |

### Erweiterte Features

| Methode | Endpunkt | Beschreibung |
| ------- | -------- | ------------ |
| `POST` | `/fhir/Patient/$validate` | Validieren ohne zu speichern |
| `GET` | `/metadata` | CapabilityStatement |

### KI-Features (brauchen `ANTHROPIC_API_KEY`)

| Methode | Endpunkt | Body | Beschreibung |
| ------- | -------- | ---- | ------------ |
| `POST` | `/fhir/Patient/$nl-search` | `{"query": "..."}` | Natürliche Sprache → FHIR-Suche |
| `POST` | `/fhir/Patient/$generate` | `{"count": 5}` | Synthetische Patienten generieren (max 50) |
| `POST` | `/fhir/$chat` | `{"message": "..."}` | KI-Chatbot mit Tool-Calling |

<details>
<summary>KI-Chat Feature Diagramm</summary>

<p align="center">
  <img src="../diagrams/ai-chat-feature.drawio.svg" alt="AI Chat Feature" width="700"/>
</p>
</details>

### Observability

| Methode | Endpunkt | Beschreibung |
| ------- | -------- | ------------ |
| `GET` | `/health` | DB-Konnektivitätscheck (`200`/`503`) |
| `GET` | `/metrics` | Prometheus Textformat |

## Konfiguration

Die gesamte Konfiguration läuft über Umgebungsvariablen:

| Variable | Pflicht | Standard | Beschreibung |
| -------- | ------- | -------- | ------------ |
| `DATABASE_URL` | Ja | `host=localhost user=postgres dbname=fhir` | PostgreSQL Connection-String |
| `BIND_ADDRESS` | Nein | `0.0.0.0:8080` | Server-Adresse |
| `API_KEY` | Nein | _(deaktiviert)_ | API-Key für `X-API-Key` Auth |
| `ANTHROPIC_API_KEY` | Nein | _(deaktiviert)_ | Aktiviert KI-Features |
| `CORS_ORIGINS` | Nein | `*` | Kommagetrennte erlaubte Origins |
| `RATE_LIMIT_RPS` | Nein | `100` | Max Requests pro Sekunde |
| `RUST_LOG` | Nein | `info` | Log-Level Filter |

## Middleware

Requests durchlaufen diese Schichten (äußerste zuerst):

1. **Prometheus Metrics** — zählt Requests, zeichnet Latenz-Histogramm auf
2. **Tracing** — HTTP-Level Tracing via `tower-http`
3. **CORS** — konfigurierbare Origin-Allowlist
4. **Request ID** — generiert oder propagiert `X-Request-ID`
5. **Audit** — loggt POST/PUT/DELETE Mutationen
6. **Rate Limit** — Token-Bucket Rate-Limiter (nur geschützte Routen)
7. **Auth** — validiert `X-API-Key` Header (nur geschützte Routen)

<p align="center">
  <img src="../diagrams/middleware-pipeline.drawio.svg" alt="Middleware Pipeline" width="600"/>
</p>

## Tests

Integrationstests nutzen [testcontainers](https://github.com/testcontainers/testcontainers-rs), um eine echte PostgreSQL-Instanz mit der PGRX Extension hochzufahren und dann alle HTTP-Endpunkte über den Axum-Router zu testen.

```bash
# Test-Datenbank-Image bauen (nur beim ersten Mal)
make test-db-image

# Alle Tests ausführen
make integration-test

# Oder direkt
cargo test -p fhir-server -- --test-threads=1
```

| Test | Was er prüft |
| ---- | ------------ |
| `test_auth` | Fehlender / falscher / korrekter API-Key |
| `test_crud_lifecycle` | Create → Read → Update → Delete → 404 |
| `test_health` | `GET /health` → 200 healthy |
| `test_history` | Create + Update → `/_history` mit 2 Versionen |
| `test_metadata` | `GET /metadata` → CapabilityStatement |
| `test_pagination` | `_count` / `_offset` + Pagination-Links |
| `test_search` | Name-, Gender-, Birthdate-Filter + kombiniert |
| `test_validate` | Valide → 200, invalide → 400 |

## CI/CD

GitHub Actions läuft bei jedem Push und PR:

| Job | Trigger | Beschreibung |
| --- | ------- | ------------ |
| **Format** | Push + PR | `cargo fmt --check` |
| **Build** | Push + PR | `cargo build` (core + server) |
| **Clippy** | nur PR | `cargo clippy -- -D warnings` |
| **Docker** | nur PR | `docker compose build` |
| **Integration** | nur PR | Test-DB-Image bauen + `cargo test` |

## Lizenz

[MIT](../LICENSE)
