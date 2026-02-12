#!/usr/bin/env bash
# FHIR Patient Server — CRUD & Search examples
#
# Usage:
#   export API_KEY="your-api-key"
#   bash examples/search.sh

set -euo pipefail

BASE="http://localhost:8080"
KEY="${API_KEY:?Set API_KEY env var first}"

echo "=== Create a Patient ==="
LOCATION=$(curl -s -D - -o /dev/null -X POST "$BASE/fhir/Patient" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $KEY" \
  -d @examples/patient.json | grep -i ^location: | tr -d '\r' | awk '{print $2}')
ID="${LOCATION##*/}"
echo "Created: $LOCATION (id=$ID)"

echo ""
echo "=== Read the Patient ==="
curl -s "$BASE/fhir/Patient/$ID" -H "X-API-Key: $KEY" | jq .

echo ""
echo "=== Update the Patient ==="
curl -s -X PUT "$BASE/fhir/Patient/$ID" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $KEY" \
  -d '{
    "resourceType": "Patient",
    "name": [{"family": "Smith", "given": ["John", "Michael"]}],
    "gender": "male",
    "birthDate": "1990-05-15",
    "active": true
  }' -w "\nHTTP %{http_code}\n"

echo ""
echo "=== Read updated Patient ==="
curl -s "$BASE/fhir/Patient/$ID" -H "X-API-Key: $KEY" | jq .

echo ""
echo "=== Search by name ==="
curl -s "$BASE/fhir/Patient?name=Smith" -H "X-API-Key: $KEY" | jq .

echo ""
echo "=== Search by gender ==="
curl -s "$BASE/fhir/Patient?gender=male" -H "X-API-Key: $KEY" | jq .

echo ""
echo "=== Search by birthdate (born after 1980) ==="
curl -s "$BASE/fhir/Patient?birthdate=ge1980-01-01" -H "X-API-Key: $KEY" | jq .

echo ""
echo "=== Paginated search ==="
curl -s "$BASE/fhir/Patient?_count=2&_offset=0" -H "X-API-Key: $KEY" | jq .

echo ""
echo "=== Sorted search (newest birthdate first) ==="
curl -s "$BASE/fhir/Patient?_sort=-birthdate&_count=5" -H "X-API-Key: $KEY" | jq .

echo ""
echo "=== Patient history ==="
curl -s "$BASE/fhir/Patient/$ID/_history" -H "X-API-Key: $KEY" | jq .

echo ""
echo "=== Validate a Patient (valid) ==="
curl -s -X POST "$BASE/fhir/Patient/\$validate" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $KEY" \
  -d '{"resourceType": "Patient", "name": [{"family": "Test"}]}' | jq .

echo ""
echo "=== Validate a Patient (invalid) ==="
curl -s -X POST "$BASE/fhir/Patient/\$validate" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $KEY" \
  -d '{"resourceType": "Observation"}' | jq .

echo ""
echo "=== Metadata (CapabilityStatement) ==="
curl -s "$BASE/metadata" | jq .

echo ""
echo "=== Health check ==="
curl -s "$BASE/health" | jq .

echo ""
echo "=== Delete the Patient ==="
curl -s -X DELETE "$BASE/fhir/Patient/$ID" -H "X-API-Key: $KEY" -w "HTTP %{http_code}\n"

echo ""
echo "Done."
