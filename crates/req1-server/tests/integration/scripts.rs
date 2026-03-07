use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, create_module, create_object, spawn_server};

#[tokio::test]
async fn test_script_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/scripts", api(&base));

    let res = client
        .post(&url)
        .json(&json!({
            "name": "Auto-classify",
            "script_type": "trigger",
            "hook_point": "pre_save",
            "source_code": "",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let s: Value = res.json().await.unwrap();
    let s_id = s["id"].as_str().unwrap();
    assert_eq!(s["name"], "Auto-classify");
    assert_eq!(s["script_type"], "trigger");
    assert_eq!(s["hook_point"], "pre_save");

    let res = client.get(format!("{url}/{s_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(list.as_array().unwrap().iter().any(|i| i["id"] == s_id));

    let res = client
        .patch(format!("{url}/{s_id}"))
        .json(&json!({"source_code": "// no-op trigger"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.delete(format!("{url}/{s_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get(format!("{url}/{s_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_script_invalid_type() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let res = client
        .post(format!("{}/modules/{mod_id}/scripts", api(&base)))
        .json(&json!({
            "name": "Bad Script",
            "script_type": "invalid",
            "source_code": "null",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_script_trigger_requires_hook_point() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let res = client
        .post(format!("{}/modules/{mod_id}/scripts", api(&base)))
        .json(&json!({
            "name": "No Hook",
            "script_type": "trigger",
            "source_code": "",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_script_test_endpoint() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/scripts", api(&base));

    let s: Value = client
        .post(&url)
        .json(&json!({
            "name": "Echo heading",
            "script_type": "layout",
            "source_code": "return obj.heading || ''",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let s_id = s["id"].as_str().unwrap();

    let res = client
        .post(format!("{url}/{s_id}/test"))
        .json(&json!({
            "object": {
                "id": "00000000-0000-0000-0000-000000000001",
                "heading": "HELLO",
                "version": 1,
            },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let result: Value = res.json().await.unwrap();
    assert_eq!(result["script_type"], "layout");
    assert_eq!(result["value"], "HELLO");
}

#[tokio::test]
async fn test_script_execute_action() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/scripts", api(&base));

    let s: Value = client
        .post(&url)
        .json(&json!({
            "name": "Print action",
            "script_type": "action",
            "source_code": "req1.print('hello from action')",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let s_id = s["id"].as_str().unwrap();

    let res = client
        .post(format!("{url}/{s_id}/execute"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let result: Value = res.json().await.unwrap();
    assert!(
        result["output"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o == "hello from action")
    );
}

#[tokio::test]
async fn test_script_batch_layout() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let _ = create_object(&client, &base, mod_id, "OBJ-A").await;
    let _ = create_object(&client, &base, mod_id, "OBJ-B").await;

    let script_url = format!("{}/modules/{mod_id}/scripts", api(&base));
    let s: Value = client
        .post(&script_url)
        .json(&json!({
            "name": "Level layout",
            "script_type": "layout",
            "source_code": "return obj.heading || 'none'",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let s_id = s["id"].as_str().unwrap();

    let res = client
        .post(format!("{script_url}/{s_id}/layout"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let result: Value = res.json().await.unwrap();
    assert_eq!(result["results"].as_array().unwrap().len(), 2);
}
