use crate::models::token::TokenData;
use sqlx::PgPool;

pub async fn get_token_data(
    pool: &PgPool,
    token_address: &str,
    token_id: &str,
) -> Result<TokenData, sqlx::Error> {
    let token_data = sqlx::query_as!(
        TokenData,
        "SELECT token_chain_id, token_id, token_address, listed_timestamp, updated_timestamp, \
                current_owner, current_price, quantity, start_amount, end_amount, start_date, end_date, broker_id \
           FROM orderbook_token WHERE token_address = $1 AND token_id = $2",
        token_address,
        token_id
    )
        .fetch_one(pool)
        .await?;

    Ok(token_data)
}
