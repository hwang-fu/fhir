#!/usr/bin/env bash
# FHIR Patient Server — AI feature examples
#
# Prerequisites:
#   - Server running with ANTHROPIC_API_KEY set
#   - API_KEY env var set
#
# Usage:
#   export API_KEY="your-api-key"
#   bash examples/chat.sh

set -euo pipefail

BASE="http://localhost:8080"
KEY="${API_KEY:?Set API_KEY env var first}"

echo "=== Generate synthetic patients ==="
curl -s -X POST "$BASE/fhir/Patient/\$generate" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $KEY" \
  -d '{"count": 5}' | jq .

echo ""
echo "=== Natural language search ==="
curl -s -X POST "$BASE/fhir/Patient/\$nl-search" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $KEY" \
  -d '{"query": "Find all male patients born after 1990"}' | jq .

echo ""
echo "=== Chat: ask about patients ==="
curl -s -X POST "$BASE/fhir/\$chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $KEY" \
  -d '{"message": "How many patients are in the system?"}' | jq .

echo ""
echo "=== Chat: specific query ==="
curl -s -X POST "$BASE/fhir/\$chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $KEY" \
  -d '{"message": "List all female patients and their birth dates"}' | jq .

echo ""
echo "Done."
