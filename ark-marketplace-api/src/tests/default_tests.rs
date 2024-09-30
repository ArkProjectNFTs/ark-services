use crate::models::default::{LastSale, LiveAuction, Trending};
use reqwest::Client;

#[tokio::test]
async fn test_last_sales() {
    let client = Client::new();

    let url = "http://localhost:8080/last-sales".to_string();
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

    // Check if the structure matches what we expect
    let _data: Vec<LastSale> =
        serde_json::from_value(data.clone()).expect("Failed to deserialize data field");
}

#[tokio::test]
async fn test_live_auctions() {
    let client = Client::new();

    let url = "http://localhost:8080/live-auctions".to_string();
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

    // Check if the structure matches what we expect
    let _data: Vec<LiveAuction> =
        serde_json::from_value(data.clone()).expect("Failed to deserialize data field");
}

#[tokio::test]
async fn test_trending() {
    let client = Client::new();

    let url = "http://localhost:8080/trending".to_string();
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

    // Check if the structure matches what we expect
    let _data: Vec<Trending> =
        serde_json::from_value(data.clone()).expect("Failed to deserialize data field");
}
