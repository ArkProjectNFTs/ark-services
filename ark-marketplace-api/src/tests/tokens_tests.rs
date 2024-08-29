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

    let body: Value = res.json().await.expect("Failed to parse response body");
    println!("{:?}", body);
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


#[tokio::test]
async fn test_get_token_trait_filters() {
    let client = Client::new();
    let address = "0x032d99485b22f2e58c8a0206d3b3bb259997ff0db70cffd25585d7dd9a5b0546";
    let filters = "traits=%7B%22Back%22%3A%5B%22Noscope%22%5D%7D";

    let url = format!(
        "http://localhost:8080/tokens/{}/filters?{}",
        address, filters
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
