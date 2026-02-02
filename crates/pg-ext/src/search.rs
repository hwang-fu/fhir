
use pgrx::prelude::*;

/// Search for FHIR resources with filtering, pagination, and sorting
///
/// # Arguments
/// * `resource_type` - The FHIR resource type (e.g., "Patient")
/// * `params` - JSONB object with search parameters:
///   - `name`: substring match on patient name (family or given)
///   - `gender`: exact match
///   - `birthdate`: date with optional prefix (eq, ge, le, gt, lt)
///   - `_count`: max results (default 10)
///   - `_offset`: skip N results (default 0)
///   - `_sort`: field to sort by, prefix with - for descending
#[pg_extern]
fn fhir_search(
    resource_type: &str,
    params: pgrx::JsonB,
) -> TableIterator<'static, (name!(id, pgrx::Uuid), name!(data, pgrx::JsonB))> {
    let params = params.0;

    // Extract pagination params
    let count = params.get("_count").and_then(|v| v.as_i64()).unwrap_or(10);
    let offset = params.get("_offset").and_then(|v| v.as_i64()).unwrap_or(0);

    // Extract sort param
    let sort_field = params
        .get("_sort")
        .and_then(|v| v.as_str())
        .unwrap_or("created_at");
    let (sort_column, sort_dir) = if let Some(field) = sort_field.strip_prefix('-') {
        (map_sort_field(field), "DESC")
    } else {
        (map_sort_field(sort_field), "ASC")
    };

    // Build dynamic query with filters
    let mut where_clauses = vec![
        "resource_type = $1".to_string(),
        "deleted_at IS NULL".to_string(),
    ];

    // Name filter (substring match on family or given name)
    if let Some(name) = params.get("name").and_then(|v| v.as_str()) {
        where_clauses.push(format!(
            "(data->'name'->0->>'family' ILIKE '%{}%' OR data->'name'->0->>'given'->0
  ILIKE '%{}%')",
            escape_like(name),
            escape_like(name)
        ));
    }

    // Gender filter (exact match)
    if let Some(gender) = params.get("gender").and_then(|v| v.as_str()) {
        where_clauses.push(format!("data->>'gender' = '{}'", escape_sql(gender)));
    }

    // Birthdate filter with prefix operators
    if let Some(birthdate) = params.get("birthdate").and_then(|v| v.as_str()) {
        if let Some(clause) = build_date_clause(birthdate) {
            where_clauses.push(clause);
        }
    }

    let query = format!(
        "SELECT id, data FROM fhir_resources WHERE {} ORDER BY {} {} LIMIT {} OFFSET {}",
        where_clauses.join(" AND "),
        sort_column,
        sort_dir,
        count,
        offset
    );

    let results: Vec<(pgrx::Uuid, pgrx::JsonB)> = Spi::connect(|client| {
        let mut results = Vec::new();
        let tup_table = client.select(&query, None, &[resource_type.into()])?;

        for row in tup_table {
            let id: pgrx::Uuid = row.get(1)?.expect("id should not be null");
            let data: pgrx::JsonB = row.get(2)?.expect("data should not be null");
            results.push((id, data));
        }

        Ok::<_, pgrx::spi::SpiError>(results)
    })
    .expect("Failed to execute search");

    TableIterator::new(results)
}

/// Map FHIR sort fields to database columns/expressions
fn map_sort_field(field: &str) -> &'static str {
    match field {
        "birthdate" | "birthDate" => "data->>'birthDate'",
        "name" => "data->'name'->0->>'family'",
        "gender" => "data->>'gender'",
        "created_at" | "_lastUpdated" => "created_at",
        _ => "created_at",
    }
}

/// Escape special characters for LIKE patterns
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
        .replace('\'', "''")
}

/// Escape single quotes for SQL strings
fn escape_sql(s: &str) -> String {
    s.replace('\'', "''")
}

/// Build date comparison clause from FHIR date prefix
/// Supports: eq (default), ge, le, gt, lt, ne
fn build_date_clause(birthdate: &str) -> Option<String> {
    let (op, date) = if birthdate.starts_with("ge") {
        (">=", &birthdate[2..])
    } else if birthdate.starts_with("le") {
        ("<=", &birthdate[2..])
    } else if birthdate.starts_with("gt") {
        (">", &birthdate[2..])
    } else if birthdate.starts_with("lt") {
        ("<", &birthdate[2..])
    } else if birthdate.starts_with("ne") {
        ("!=", &birthdate[2..])
    } else if birthdate.starts_with("eq") {
        ("=", &birthdate[2..])
    } else {
        ("=", birthdate)
    };

    // Validate date format (basic check)
    if date.is_empty() {
        return None;
    }

    Some(format!("data->>'birthDate' {} '{}'", op, escape_sql(date)))
}
