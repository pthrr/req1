use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, spawn_server};

#[tokio::test]
async fn test_app_user_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let url = format!("{}/users", api(&base));

    let email = format!("user-{}@test.com", uuid::Uuid::now_v7());
    let res = client
        .post(&url)
        .json(&json!({
            "email": email,
            "display_name": "Test User",
            "role": "editor",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let user: Value = res.json().await.unwrap();
    let user_id = user["id"].as_str().unwrap();
    assert_eq!(user["display_name"], "Test User");
    assert_eq!(user["role"], "editor");

    let res = client.get(format!("{url}/{user_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == user_id)
    );

    let res = client
        .patch(format!("{url}/{user_id}"))
        .json(&json!({"display_name": "Updated User"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["display_name"], "Updated User");

    let res = client
        .delete(format!("{url}/{user_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get(format!("{url}/{user_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_app_user_invalid_role() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;

    let res = client
        .post(format!("{}/users", api(&base)))
        .json(&json!({
            "email": "bad@test.com",
            "display_name": "Bad Role",
            "role": "superadmin",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_app_user_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/users/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
