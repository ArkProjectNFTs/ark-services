use crate::models::collection::CollectionFullData;
use reqwest::Client;
use serde_json::Value;

const ADDRESS: &str = "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af";
const CHAIN_ID: &str = "0x534e5f4d41494e";
const USER_ADDRESS: &str = "0xe4769a4d2f7f69c70951a003eba5c32707cef3cdfb6b27ca63567f51cdd078";

#[tokio::test]
async fn test_get_collections() {
    let client = Client::new();

    let url = "http://localhost:8080/collections";
    let res = client
        .get(url)
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
    let _offers_data: Vec<CollectionFullData> =
        serde_json::from_value(data.clone()).expect("Failed to deserialize data field");
}

#[tokio::test]
async fn test_get_collection_activity() {
    let client = Client::new();

    let url = format!("http://localhost:8080/collections/{}/activity", ADDRESS);
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
async fn test_get_collection() {
    let client = Client::new();

    let url = format!("http://localhost:8080/collections/{}/{}", ADDRESS, CHAIN_ID);
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
async fn test_get_portfolio_collections() {
    let client = Client::new();

    let url = format!(
        "http://localhost:8080/portfolio/{}/collections",
        USER_ADDRESS
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
async fn test_search_collections() {
    let client = Client::new();

    let url = "http://localhost:8080/collections/search";
    let res = client
        .get(url)
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
