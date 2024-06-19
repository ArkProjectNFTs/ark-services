use tracing::{error, info};
use sqlx::PgPool;

pub async fn update_listed_tokens(pool: &PgPool) {
    info!("Updating listed tokens...");

    let update_is_listed_query = r#"
        UPDATE token
        SET is_listed = CASE
            WHEN NOW() BETWEEN to_timestamp(listing_start_date) AND to_timestamp(listing_end_date) THEN true
            ELSE false
        END
        WHERE listing_start_date IS NOT NULL AND listing_end_date IS NOT NULL;
    "#;

    match sqlx::query(update_is_listed_query)
        .execute(pool)
        .await
    {
        Ok(_) => info!("Update of is_listed field successful."),
        Err(e) => error!("Failed to update is_listed field: {}", e),
    }

    let clean_dates_query = r#"
        UPDATE token
        SET listing_start_date = NULL,
            listing_end_date = NULL
        WHERE (NOW() > to_timestamp(listing_end_date) OR NOW() < to_timestamp(listing_start_date))
          AND listing_start_date IS NOT NULL AND listing_end_date IS NOT NULL;
    "#;

    match sqlx::query(clean_dates_query)
        .execute(pool)
        .await
    {
        Ok(_) => info!("Cleanup of listing dates successful."),
        Err(e) => error!("Failed to clean up listing dates: {}", e),
    }
}
