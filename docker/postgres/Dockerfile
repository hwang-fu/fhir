# ---- Stage 1: Build the PGRX extension ----
FROM rust:1.92-bookworm AS builder

# Install PostgreSQL 17 server + dev headers and build tools
RUN apt-get update && apt-get install -y \
    postgresql-common \
    && /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y \
    && apt-get update && apt-get install -y \
    postgresql-server-dev-17 \
    postgresql-17 \
    pkg-config \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-pgrx matching the project version
RUN cargo install cargo-pgrx --version "~0.16" --locked

# Initialize pgrx with the system PostgreSQL 17
RUN cargo pgrx init --pg17=/usr/lib/postgresql/17/bin/pg_config

WORKDIR /build
COPY . .

# Package the extension for PG 17
RUN cargo pgrx package --pg-config /usr/lib/postgresql/17/bin/pg_config -p fhir-pg-ext

# ---- Stage 2: PostgreSQL with extension installed ----
FROM postgres:17-bookworm

# Copy compiled extension artifacts from builder
COPY --from=builder /build/target/release/fhir_pg_ext-pg17/usr/share/postgresql/17/extension/ \
    /usr/share/postgresql/17/extension/
COPY --from=builder /build/target/release/fhir_pg_ext-pg17/usr/lib/postgresql/17/lib/ \
    /usr/lib/postgresql/17/lib/

# Auto-create the extension on first startup
COPY docker/postgres/init.sql /docker-entrypoint-initdb.d/
