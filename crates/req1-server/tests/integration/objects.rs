use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, create_module, create_object, spawn_server};

#[tokio::test]
async fn test_object_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"heading": "REQ-001", "body": "Shall do something"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let obj: Value = res.json().await.unwrap();
    let obj_id = obj["id"].as_str().unwrap();
    assert_eq!(obj["heading"], "REQ-001");
    assert_eq!(obj["current_version"], 1);

    let res = client.get(format!("{url}/{obj_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    let res = client
        .patch(format!("{url}/{obj_id}"))
        .json(&json!({"heading": "REQ-001-updated", "body": "Updated body"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["heading"], "REQ-001-updated");
    assert_eq!(updated["current_version"], 2);

    let res = client
        .get(format!("{url}/{obj_id}/history"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let history: Value = res.json().await.unwrap();
    assert_eq!(history["items"].as_array().unwrap().len(), 2);

    let res = client
        .delete(format!("{url}/{obj_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client
        .get(format!("{url}/{obj_id}/history"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let history_after: Value = res.json().await.unwrap();
    assert_eq!(history_after["items"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_object_filters() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    let _ = client
        .post(&url)
        .json(&json!({"heading": "REQ-ALPHA", "body": "Alpha requirement"}))
        .send()
        .await
        .unwrap();
    let _ = client
        .post(&url)
        .json(&json!({"heading": "REQ-BETA", "body": "Beta requirement"}))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{url}?heading=ALPHA"))
        .send()
        .await
        .unwrap();
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["heading"], "REQ-ALPHA");

    let res = client.get(format!("{url}?body=Beta")).send().await.unwrap();
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    let res = client
        .get(format!("{url}?sort_by=heading&sort_dir=desc"))
        .send()
        .await
        .unwrap();
    let list: Value = res.json().await.unwrap();
    let items = list["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["heading"], "REQ-BETA");
    assert_eq!(items[1]["heading"], "REQ-ALPHA");
}

#[tokio::test]
async fn test_object_attribute_validation() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let attr_url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));
    let obj_url = format!("{}/modules/{mod_id}/objects", api(&base));

    let _ = client
        .post(&attr_url)
        .json(&json!({
            "name": "priority",
            "data_type": "enum",
            "enum_values": ["low", "medium", "high"]
        }))
        .send()
        .await
        .unwrap();

    let _ = client
        .post(&attr_url)
        .json(&json!({"name": "weight", "data_type": "integer"}))
        .send()
        .await
        .unwrap();

    let res = client
        .post(&obj_url)
        .json(&json!({
            "heading": "REQ-VAL",
            "attributes": {"priority": "high", "weight": 42}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = client
        .post(&obj_url)
        .json(&json!({
            "heading": "REQ-BAD",
            "attributes": {"priority": "critical"}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = client
        .post(&obj_url)
        .json(&json!({
            "heading": "REQ-BAD2",
            "attributes": {"weight": "heavy"}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = client
        .post(&obj_url)
        .json(&json!({
            "heading": "REQ-BAD3",
            "attributes": {"nonexistent": "val"}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_object_type_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/object-types", api(&base));

    let res = client
        .post(&url)
        .json(&json!({
            "module_id": mod_id,
            "name": "Functional Requirement",
            "required_attributes": ["priority"],
            "attribute_schema": {},
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let ot: Value = res.json().await.unwrap();
    let ot_id = ot["id"].as_str().unwrap();
    assert_eq!(ot["name"], "Functional Requirement");

    let res = client.get(format!("{url}/{ot_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .get(format!("{url}?module_id={mod_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == ot_id)
    );

    let res = client
        .patch(format!("{url}/{ot_id}"))
        .json(&json!({"name": "Non-Functional Requirement"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Non-Functional Requirement");

    let res = client
        .delete(format!("{url}/{ot_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get(format!("{url}/{ot_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_object_type_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/object-types/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_comment_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let obj = create_object(&client, &base, mod_id, "REQ-COMMENT").await;
    let obj_id = obj["id"].as_str().unwrap();
    let url = format!("{}/objects/{obj_id}/comments", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"body": "This needs review"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let comment: Value = res.json().await.unwrap();
    let comment_id = comment["id"].as_str().unwrap();
    assert_eq!(comment["body"], "This needs review");
    assert_eq!(comment["resolved"], false);

    let res = client
        .get(format!("{url}/{comment_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    let res = client
        .patch(format!("{url}/{comment_id}"))
        .json(&json!({"body": "Updated comment"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["body"], "Updated comment");

    let res = client
        .patch(format!("{url}/{comment_id}"))
        .json(&json!({"resolved": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let resolved: Value = res.json().await.unwrap();
    assert_eq!(resolved["resolved"], true);

    let res = client
        .delete(format!("{url}/{comment_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client
        .get(format!("{url}/{comment_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_comment_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let obj = create_object(&client, &base, mod_id, "REQ-COMMENT-NF").await;
    let obj_id = obj["id"].as_str().unwrap();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!(
            "{}/objects/{obj_id}/comments/{fake_id}",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
