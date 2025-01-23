use anyhow::Result;
use sqlx::PgPool;
use tracing::{error, info};

#[derive(sqlx::FromRow, Debug)]
struct Currency {
    contract_address: String,
}

pub async fn update_currency_prices(pool: &PgPool) {
    let currencies = match fetch_currencies(pool).await {
        Ok(currencies) => currencies,
        Err(e) => {
            error!("Failed to fetch currencies: {}", e);
            return;
        }
    };

    for currency in currencies {
        match update_currency_price(pool, &currency).await {
            Ok(_) => info!("Updated prices for currency {}", currency.contract_address),
            Err(e) => error!(
                "Failed to update prices for currency {}: {}",
                currency.contract_address, e
            ),
        }
    }
}

async fn update_currency_price(pool: &PgPool, currency: &Currency) -> Result<()> {
    let price_info =
        crate::tasks::token_price::fetch_token_price_from_avnu(&currency.contract_address).await?;

    sqlx::query(
        "UPDATE currency SET price_in_usd = $1, price_in_eth = $2 WHERE contract_address = $3",
    )
    .bind(price_info.price_in_usd)
    .bind(price_info.price_in_eth)
    .bind(&currency.contract_address)
    .execute(pool)
    .await?;

    Ok(())
}

async fn fetch_currencies(pool: &PgPool) -> Result<Vec<Currency>, sqlx::Error> {
    sqlx::query_as::<_, Currency>("SELECT contract_address FROM currency")
        .fetch_all(pool)
        .await
}
