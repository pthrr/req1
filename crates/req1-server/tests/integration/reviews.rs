use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, create_module, spawn_server};

#[tokio::test]
async fn test_review_package_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/review-packages", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "Review Package 1", "description": "First review"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let rp: Value = res.json().await.unwrap();
    let rp_id = rp["id"].as_str().unwrap();
    assert_eq!(rp["name"], "Review Package 1");
    assert_eq!(rp["status"], "draft");

    let res = client.get(format!("{url}/{rp_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == rp_id)
    );

    let res = client
        .patch(format!("{url}/{rp_id}"))
        .json(&json!({"name": "Updated Package"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Updated Package");

    let res = client
        .delete(format!("{url}/{rp_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get(format!("{url}/{rp_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_review_package_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/review-packages/{fake_id}",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_review_assignment_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let rp: Value = client
        .post(format!("{}/modules/{mod_id}/review-packages", api(&base)))
        .json(&json!({"name": "RP for assignments"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rp_id = rp["id"].as_str().unwrap();
    let url = format!("{}/review-packages/{rp_id}/assignments", api(&base));

    let res = client.post(&url).json(&json!({})).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let ra: Value = res.json().await.unwrap();
    let ra_id = ra["id"].as_str().unwrap();
    assert_eq!(ra["status"], "pending");

    let res = client.get(format!("{url}/{ra_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    let res = client
        .patch(format!("{url}/{ra_id}"))
        .json(&json!({"status": "approved", "comment": "Looks good"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["status"], "approved");
    assert!(updated["signed_at"].as_str().is_some());

    let res = client
        .delete(format!("{url}/{ra_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_change_proposal_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/change-proposals", api(&base));

    let res = client
        .post(&url)
        .json(&json!({
            "title": "Update safety requirements",
            "description": "Need to add safety constraints",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let cp: Value = res.json().await.unwrap();
    let cp_id = cp["id"].as_str().unwrap();
    assert_eq!(cp["title"], "Update safety requirements");
    assert_eq!(cp["status"], "draft");

    let res = client.get(format!("{url}/{cp_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == cp_id)
    );

    let res = client
        .patch(format!("{url}/{cp_id}"))
        .json(&json!({"title": "Updated title", "status": "submitted"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["title"], "Updated title");
    assert_eq!(updated["status"], "submitted");

    let res = client
        .delete(format!("{url}/{cp_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get(format!("{url}/{cp_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_change_proposal_invalid_status() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/change-proposals", api(&base));

    let cp: Value = client
        .post(&url)
        .json(&json!({"title": "CP for bad status"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cp_id = cp["id"].as_str().unwrap();

    let res = client
        .patch(format!("{url}/{cp_id}"))
        .json(&json!({"status": "bogus"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
