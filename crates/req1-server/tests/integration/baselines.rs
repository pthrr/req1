use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, create_module, spawn_server};

#[tokio::test]
async fn test_baseline_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let obj_url = format!("{}/modules/{mod_id}/objects", api(&base));
    let bl_url = format!("{}/modules/{mod_id}/baselines", api(&base));

    let obj: Value = client
        .post(&obj_url)
        .json(&json!({"heading": "REQ-BL-001"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let obj_id = obj["id"].as_str().unwrap();

    let res = client
        .post(&bl_url)
        .json(&json!({"name": "v1.0"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bl: Value = res.json().await.unwrap();
    let bl_id = bl["id"].as_str().unwrap();
    assert_eq!(bl["name"], "v1.0");
    assert_eq!(bl["locked"], true);
    assert_eq!(bl["entries"].as_array().unwrap().len(), 1);
    assert_eq!(bl["entries"][0]["object_id"], obj_id);
    assert_eq!(bl["entries"][0]["version"], 1);

    let res = client
        .get(format!("{bl_url}/{bl_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&bl_url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == bl_id)
    );

    let res = client
        .delete(format!("{bl_url}/{bl_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_baseline_diff() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let obj_url = format!("{}/modules/{mod_id}/objects", api(&base));
    let bl_url = format!("{}/modules/{mod_id}/baselines", api(&base));

    let obj: Value = client
        .post(&obj_url)
        .json(&json!({"heading": "REQ-DIFF"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let obj_id = obj["id"].as_str().unwrap();

    let bl_a: Value = client
        .post(&bl_url)
        .json(&json!({"name": "baseline-a"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let bl_a_id = bl_a["id"].as_str().unwrap();

    let _ = client
        .patch(format!("{obj_url}/{obj_id}"))
        .json(&json!({"heading": "REQ-DIFF-v2"}))
        .send()
        .await
        .unwrap();

    let _ = client
        .post(&obj_url)
        .json(&json!({"heading": "REQ-NEW"}))
        .send()
        .await
        .unwrap();

    let bl_b: Value = client
        .post(&bl_url)
        .json(&json!({"name": "baseline-b"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let bl_b_id = bl_b["id"].as_str().unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/baseline-diff?a={bl_a_id}&b={bl_b_id}",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let diff: Value = res.json().await.unwrap();

    assert_eq!(diff["added"].as_array().unwrap().len(), 1);
    assert_eq!(diff["modified"].as_array().unwrap().len(), 1);
    assert_eq!(diff["removed"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_baseline_set_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let url = format!("{}/baseline-sets", api(&base));

    let res = client
        .post(&url)
        .json(&json!({
            "name": "Release 1.0",
            "version": "1.0.0",
            "description": "First release baseline set",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bs: Value = res.json().await.unwrap();
    let bs_id = bs["id"].as_str().unwrap();
    assert_eq!(bs["name"], "Release 1.0");
    assert_eq!(bs["version"], "1.0.0");

    let res = client.get(format!("{url}/{bs_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == bs_id)
    );

    let res = client
        .patch(format!("{url}/{bs_id}"))
        .json(&json!({"name": "Release 1.1", "version": "1.1.0"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Release 1.1");
    assert_eq!(updated["version"], "1.1.0");

    let res = client
        .delete(format!("{url}/{bs_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get(format!("{url}/{bs_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
