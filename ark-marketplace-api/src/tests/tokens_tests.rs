use crate::models::token::TokenMarketData;
use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn test_get_tokens() {
    let client = Client::new();
    let address = "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af";
    let chain_id = "0x534e5f4d41494e";

    let url = format!(
        "http://localhost:8080/collections/{}/{}/tokens",
        address, chain_id
    );
    let res = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");

    assert!(
        res.status().is_success(),
        "Request failed with status: {}",
        res.status()
    );

    let body: Value = res.json().await.expect("Failed to parse response body");
    println!("{:?}", body);
}

#[tokio::test]
async fn test_get_token() {
    let client = Client::new();
    let address = "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af";
    let chain_id = "0x534e5f4d41494e";
    let token_id = "445743458073";

    let url = format!(
        "http://localhost:8080/tokens/{}/{}/{}",
        address, chain_id, token_id
    );
    let res = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");

    assert!(
        res.status().is_success(),
        "Request failed with status: {}",
        res.status()
    );

    let body: Value = res.json().await.expect("Failed to parse response body");
    println!("{:?}", body);
}

#[tokio::test]
async fn test_get_token_market() {
    let client = Client::new();
    let address = "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af";
    let chain_id = "0x534e5f4d41494e";
    let token_id = "445743458073";

    let url = format!(
        "http://localhost:8080/tokens/{}/{}/{}/marketdata",
        address, chain_id, token_id
    );
    let res = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");

    assert!(
        res.status().is_success(),
        "Request failed with status: {}",
        res.status()
    );

    let body: serde_json::Value = res.json().await.expect("Failed to parse response body");
    let data = &body["data"];
    println!("{:?}", data);
    // Check if the structure matches what we expect
    let _data: TokenMarketData =
        serde_json::from_value(data.clone()).expect("Failed to deserialize data field");
}

#[tokio::test]
async fn test_get_token_offers() {
    let client = Client::new();
    let address = "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af";
    let chain_id = "0x534e5f4d41494e";
    let token_id = "445743458073";

    let url = format!(
        "http://localhost:8080/tokens/{}/{}/{}/offers",
        address, chain_id, token_id
    );
    let res = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");

    assert!(
        res.status().is_success(),
        "Request failed with status: {}",
        res.status()
    );

    let body: Value = res.json().await.expect("Failed to parse response body");
    println!("{:?}", body);
}

#[tokio::test]
async fn test_get_token_activity() {
    let client = Client::new();
    let address = "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af";
    let chain_id = "0x534e5f4d41494e";
    let token_id = "445743458073";

    let url = format!(
        "http://localhost:8080/tokens/{}/{}/{}/activity",
        address, chain_id, token_id
    );
    let res = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");

    assert!(
        res.status().is_success(),
        "Request failed with status: {}",
        res.status()
    );

    let body: Value = res.json().await.expect("Failed to parse response body");
    println!("{:?}", body);
}
