-- FHIR Resources table: stores current version of each resource
CREATE TABLE IF NOT EXISTS fhir_resources (
    id              UUID PRIMARY KEY,
    resource_type   TEXT NOT NULL,
    version         INTEGER NOT NULL DEFAULT 1,
    data            JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ  -- NULL means not deleted (soft delete)
);

-- FHIR History table: stores all versions of resources
CREATE TABLE IF NOT EXISTS fhir_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id     UUID NOT NULL,
    resource_type   TEXT NOT NULL,
    version         INTEGER NOT NULL,
    data            JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (resource_id, version)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_fhir_resources_type
    ON fhir_resources(resource_type);

CREATE INDEX IF NOT EXISTS idx_fhir_resources_type_deleted
    ON fhir_resources(resource_type) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_fhir_history_resource_id
    ON fhir_history(resource_id);

CREATE INDEX IF NOT EXISTS idx_fhir_history_resource_version
    ON fhir_history(resource_id, version DESC);
