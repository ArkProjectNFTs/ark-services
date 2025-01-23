use reqwest::Client;
use serde::Deserialize;
use std::env;

const DEFAULT_PRICE: f64 = 0.0;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AvnuTokenPriceResponse {
    pub price_in_usd: Option<f64>,
    pub price_in_eth: Option<f64>,
}

#[derive(Debug)]
pub struct TokenPriceInfo {
    pub price_in_usd: f64,
    pub price_in_eth: f64,
}

impl TokenPriceInfo {
    fn new(price_in_usd: Option<f64>, price_in_eth: Option<f64>) -> Self {
        Self {
            price_in_usd: price_in_usd.unwrap_or(DEFAULT_PRICE),
            price_in_eth: price_in_eth.unwrap_or(DEFAULT_PRICE),
        }
    }

    fn default() -> Self {
        Self {
            price_in_usd: DEFAULT_PRICE,
            price_in_eth: DEFAULT_PRICE,
        }
    }
}

pub async fn fetch_token_price_from_avnu(
    token_address: &str,
) -> Result<TokenPriceInfo, reqwest::Error> {
    let client = Client::new();
    let avnu_api_url = format!(
        "{}/v1/tokens/prices?token={}",
        env::var("AVNU_API_BASE_URL")
            .unwrap_or_else(|_| "https://starknet.impulse.avnu.fi".to_string()),
        token_address
    );

    let response = client
        .get(&avnu_api_url)
        .header("accept", "*/*")
        .send()
        .await?;

    let price_response = response.json::<Vec<AvnuTokenPriceResponse>>().await?;

    Ok(price_response
        .first()
        .map(|token_price_data| {
            TokenPriceInfo::new(token_price_data.price_in_usd, token_price_data.price_in_eth)
        })
        .unwrap_or_else(TokenPriceInfo::default))
}
