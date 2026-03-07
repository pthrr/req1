use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, create_module, spawn_server};

#[tokio::test]
async fn test_attribute_definition_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "priority", "data_type": "string"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let attr: Value = res.json().await.unwrap();
    let attr_id = attr["id"].as_str().unwrap();
    assert_eq!(attr["name"], "priority");
    assert_eq!(attr["data_type"], "string");

    let res = client.get(format!("{url}/{attr_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == attr_id)
    );

    let res = client
        .patch(format!("{url}/{attr_id}"))
        .json(&json!({"name": "importance"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "importance");

    let res = client
        .delete(format!("{url}/{attr_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_attribute_definition_enum_type() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    let res = client
        .post(&url)
        .json(&json!({
            "name": "status",
            "data_type": "enum",
            "enum_values": ["draft", "approved", "rejected"],
            "default_value": "draft"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let attr: Value = res.json().await.unwrap();
    assert_eq!(attr["data_type"], "enum");
    assert_eq!(attr["default_value"], "draft");
}

#[tokio::test]
async fn test_attribute_definition_enum_requires_values() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "status", "data_type": "enum"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_attribute_definition_invalid_type() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "x", "data_type": "foobar"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_attribute_definition_invalid_default() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "count", "data_type": "integer", "default_value": "abc"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = client
        .post(&url)
        .json(&json!({"name": "flag", "data_type": "bool", "default_value": "maybe"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_attribute_definition_enum_default_not_in_values() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    let res = client
        .post(&url)
        .json(&json!({
            "name": "status",
            "data_type": "enum",
            "enum_values": ["a", "b"],
            "default_value": "c"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_enum_values_non_array_rejected() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "x", "data_type": "string", "enum_values": ["a", "b"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
