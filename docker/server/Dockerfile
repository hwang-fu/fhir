# ---- Stage 1: Build the server binary ----
FROM rust:1.92-bookworm AS builder

WORKDIR /build
COPY . .

# Build the server crate (debug mode to avoid fhir-model MIR cycle bug E0391)
RUN cargo build -p fhir-server

# ---- Stage 2: Minimal runtime image ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/debug/fhir-server /usr/local/bin/fhir-server

EXPOSE 8080

CMD ["fhir-server"]
