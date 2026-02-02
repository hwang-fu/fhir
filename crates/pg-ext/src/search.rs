
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
