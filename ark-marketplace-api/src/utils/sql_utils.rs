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
        ("owner", "asc", Some(value)) if !value.is_empty() => format!(
            "CASE
                WHEN token.current_owner = '{}' THEN 0
                ELSE 1
            END, NULLIF(token.current_owner, '') ASC NULLS LAST, CAST(token.token_id AS NUMERIC) ASC",
            value
        ),
        ("owner", "desc", Some(value)) if !value.is_empty() => format!(
            "CASE
                WHEN token.current_owner = '{}' THEN 0
                ELSE 1
            END, NULLIF(token.current_owner, '') DESC NULLS LAST, CAST(token.token_id AS NUMERIC) DESC",
            value
        ),
        (_, "asc", _) => "CAST(token.token_id AS NUMERIC) ASC".to_string(),
        (_, "desc", _) => "CAST(token.token_id AS NUMERIC) DESC".to_string(),
        _ => "CAST(token.token_id AS NUMERIC) ASC".to_string(),
    }
}

pub fn generate_order_by_clause_collections(sort: &str, direction: &str) -> String {
    let order_by_clause_collections = if sort == "floor_price" {
        format!("ORDER BY floor_price {} NULLS LAST", direction)
    } else if sort == "floor_percentage" {
        format!(
            "ORDER BY contract_marketdata.floor_percentage {} NULLS LAST",
            direction
        )
    } else if sort == "volume" {
        format!(
            "ORDER BY contract_marketdata.volume {} NULLS LAST",
            direction
        )
    } else if sort == "top_bid" {
        format!("ORDER BY top_bid {} NULLS LAST", direction)
    } else if sort == "number_of_sales" {
        format!(
            "ORDER BY contract_marketdata.number_of_sales {} NULLS LAST",
            direction
        )
    } else if sort == "marketcap" {
        format!("ORDER BY marketcap {} NULLS LAST", direction)
    } else if sort == "listed" {
        format!("ORDER BY token_listed_count {} NULLS LAST", direction)
    } else {
        String::new()
    };

    order_by_clause_collections
}
