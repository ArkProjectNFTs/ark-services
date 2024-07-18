use serde::Deserialize;

pub const CHAIN_ID: &str = "0x534e5f4d41494e";


#[derive(Deserialize)]
pub struct PageParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
}

pub fn extract_page_params(
    query_string: &str,
    default_page: i64,
    default_items_per_page: i64,
) -> Result<(i64, i64), String> {
    match serde_qs::from_str::<PageParameters>(query_string) {
        Err(e) => {
            let msg = format!("Error when parsing page query parameters: {}", e);
            tracing::error!(msg);
            Err(msg)
        }
        Ok(params) => Ok((
            params.page.unwrap_or(default_page),
            params.items_per_page.unwrap_or(default_items_per_page),
        )),
    }
}
