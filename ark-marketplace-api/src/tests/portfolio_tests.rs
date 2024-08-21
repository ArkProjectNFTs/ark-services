use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn test_get_activity() {
    let client = Client::new();
    let user_address = "0xe4769a4d2f7f69c70951a003eba5c32707cef3cdfb6b27ca63567f51cdd078";

    let url = format!("http://localhost:8080/portfolio/{}/activity", user_address);
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
async fn test_get_tokens_portfolio() {
    let client = Client::new();
    let user_address = "0xe4769a4d2f7f69c70951a003eba5c32707cef3cdfb6b27ca63567f51cdd078";

    let url = format!("http://localhost:8080/portfolio/{}", user_address);
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
