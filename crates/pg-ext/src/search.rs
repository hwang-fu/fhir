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
