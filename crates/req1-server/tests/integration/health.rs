use axum::http::StatusCode;
use reqwest::Client;
use serde_json::Value;

use super::common::spawn_server;

#[tokio::test]
async fn test_health_liveness() {
    let base = spawn_server().await;
    let client = Client::new();

    let res = client
        .get(format!("{base}/health/live"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_health_readiness() {
    let base = spawn_server().await;
    let client = Client::new();

    let res = client
        .get(format!("{base}/health/ready"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}
