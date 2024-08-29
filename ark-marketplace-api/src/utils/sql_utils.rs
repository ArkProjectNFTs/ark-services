
pub fn generate_order_by_clause(
    sort_field: &str,
    sort_direction: &str,
    sort_value: Option<&str>,
) -> String {
    match (sort_field, sort_direction, sort_value) {
        ("price", "asc", _) => {
            "token.listing_start_amount ASC NULLS LAST, CAST(token.token_id AS NUMERIC)".to_string()
        }
        ("price", "desc", _) => {
            "token.listing_start_amount DESC NULLS FIRST, CAST(token.token_id AS NUMERIC)".to_string()
        }
        ("owner", "asc", Some(value)) => format!(
            "CASE WHEN token.current_owner = '{}' THEN 0 ELSE 1 END, token.current_owner ASC, CAST(token.token_id AS NUMERIC) ASC",
            value
        ),
        ("owner", "desc", Some(value)) => format!(
            "CASE WHEN token.current_owner = '{}' THEN 0 ELSE 1 END, token.current_owner DESC, CAST(token.token_id AS NUMERIC) DESC",
            value
        ),
        (_, "asc", _) => "CAST(token.token_id AS NUMERIC) ASC".to_string(),
        (_, "desc", _) => "CAST(token.token_id AS NUMERIC) DESC".to_string(),
        _ => "CAST(token.token_id AS NUMERIC) ASC".to_string(),
    }
}
