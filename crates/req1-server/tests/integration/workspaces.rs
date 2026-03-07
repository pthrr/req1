use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, create_workspace, spawn_server};

#[tokio::test]
async fn test_workspace_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let url = format!("{}/workspaces", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "Test Workspace"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let ws: Value = res.json().await.unwrap();
    let ws_id = ws["id"].as_str().unwrap();
    assert_eq!(ws["name"], "Test Workspace");

    let res = client.get(format!("{url}/{ws_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got: Value = res.json().await.unwrap();
    assert_eq!(got["id"], ws_id);

    let res = client.get(format!("{url}?limit=500")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == ws_id)
    );

    let res = client
        .patch(format!("{url}/{ws_id}"))
        .json(&json!({"name": "Renamed Workspace"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed Workspace");

    let res = client
        .delete(format!("{url}/{ws_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get(format!("{url}/{ws_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_workspace_get_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/workspaces/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_workspace_delete_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .delete(format!("{}/workspaces/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_project_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let url = format!("{}/workspaces/{ws_id}/projects", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "Test Project"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let proj: Value = res.json().await.unwrap();
    let proj_id = proj["id"].as_str().unwrap();
    assert_eq!(proj["name"], "Test Project");
    assert_eq!(proj["workspace_id"], ws_id);

    let res = client.get(format!("{url}/{proj_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == proj_id)
    );

    let res = client
        .patch(format!("{url}/{proj_id}"))
        .json(&json!({"name": "Renamed Project"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed Project");

    let res = client
        .delete(format!("{url}/{proj_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get(format!("{url}/{proj_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_pagination() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let url = format!("{}/workspaces", api(&base));

    for i in 0..5 {
        let _ = client
            .post(&url)
            .json(&json!({"name": format!("paginated-ws-{i}")}))
            .send()
            .await
            .unwrap();
    }

    let res = client
        .get(format!("{url}?limit=2&offset=0"))
        .send()
        .await
        .unwrap();
    let page: Value = res.json().await.unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["limit"], 2);
    assert_eq!(page["offset"], 0);
    assert!(page["total"].as_u64().unwrap() >= 5);
}

#[tokio::test]
async fn test_workspace_cascade_deletes_project() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();

    let proj: Value = client
        .post(format!("{}/workspaces/{ws_id}/projects", api(&base)))
        .json(&json!({"name": "will-be-cascaded"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let proj_id = proj["id"].as_str().unwrap();

    let res = client
        .delete(format!("{}/workspaces/{ws_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client
        .get(format!(
            "{}/workspaces/{ws_id}/projects/{proj_id}",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
