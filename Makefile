.PHONY: build test fmt clippy up down logs health seed clean

# Build the core and server crates
build:
	cargo build -p fhir-core -p fhir-server

# Run cargo tests (core and server only; pg-ext requires pgrx test harness)
test:
	cargo test -p fhir-core -p fhir-server

# Check formatting
fmt:
	cargo fmt --check -p fhir-core -p fhir-server

# Run clippy lints
clippy:
	cargo clippy -p fhir-core -p fhir-server -- -D warnings

# Start all services via Docker Compose
up:
	docker compose up -d --build

# Stop all services
down:
	docker compose down

# Tail service logs
logs:
	docker compose logs -f

# Check server health (Docker)
health:
	@curl -sf http://localhost:8080/health | jq . || echo "Server not reachable"

# Generate seed data via AI (requires ANTHROPIC_API_KEY and running server)
seed:
	@curl -sf -X POST http://localhost:8080/fhir/Patient/$$generate \
		-H "Content-Type: application/json" \
		-H "X-API-Key: $${API_KEY:-}" \
		-d '{"count": 10}' | jq . || echo "Failed — is the server running with ANTHROPIC_API_KEY set?"

# Remove build artifacts and Docker volumes
clean:
	cargo clean
	docker compose down -v
