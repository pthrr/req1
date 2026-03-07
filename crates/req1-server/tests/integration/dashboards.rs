use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{
    api, authed_client, create_dashboard, create_module, create_object, create_workspace,
    spawn_server,
};

#[tokio::test]
async fn test_dashboard_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let url = format!("{}/workspaces/{ws_id}/dashboards", api(&base));

    // Create
    let res = client
        .post(&url)
        .json(&json!({"name": "Test Dashboard", "description": "desc"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let dash: Value = res.json().await.unwrap();
    let dash_id = dash["id"].as_str().unwrap();
    assert_eq!(dash["name"], "Test Dashboard");
    assert_eq!(dash["workspace_id"], ws_id);

    // Get
    let res = client.get(format!("{url}/{dash_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got: Value = res.json().await.unwrap();
    assert_eq!(got["id"], dash_id);
    assert_eq!(got["description"], "desc");

    // Update
    let res = client
        .patch(format!("{url}/{dash_id}"))
        .json(&json!({"name": "Updated"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Updated");

    // List
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Vec<Value> = res.json().await.unwrap();
    assert!(list.iter().any(|i| i["id"] == dash_id));

    // Delete
    let res = client
        .delete(format!("{url}/{dash_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{dash_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_dashboard_widget_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let dash = create_dashboard(&client, &base, ws_id).await;
    let dash_id = dash["id"].as_str().unwrap();
    let url = format!("{}/dashboards/{dash_id}/widgets", api(&base));

    // Create widget
    let res = client
        .post(&url)
        .json(&json!({
            "widget_type": "coverage_chart",
            "title": "Coverage",
            "position_x": 0,
            "position_y": 0,
            "width": 6,
            "height": 4
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let widget: Value = res.json().await.unwrap();
    let wid = widget["id"].as_str().unwrap();
    assert_eq!(widget["widget_type"], "coverage_chart");
    assert_eq!(widget["title"], "Coverage");
    assert_eq!(widget["dashboard_id"], dash_id);
    assert_eq!(widget["width"], 6);
    assert_eq!(widget["height"], 4);

    // Get widget
    let res = client.get(format!("{url}/{wid}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got: Value = res.json().await.unwrap();
    assert_eq!(got["id"], wid);

    // Update widget
    let res = client
        .patch(format!("{url}/{wid}"))
        .json(&json!({"title": "Updated Coverage"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["title"], "Updated Coverage");

    // List widgets
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Vec<Value> = res.json().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], wid);

    // Delete widget
    let res = client.delete(format!("{url}/{wid}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{wid}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_dashboard_widget_invalid_type() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let dash = create_dashboard(&client, &base, ws_id).await;
    let dash_id = dash["id"].as_str().unwrap();

    let res = client
        .post(format!("{}/dashboards/{dash_id}/widgets", api(&base)))
        .json(&json!({
            "widget_type": "invalid_type",
            "title": "Bad Widget"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_dashboard_widget_data_coverage() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    // Create some objects
    let _ = create_object(&client, &base, mod_id, "Req A").await;
    let _ = create_object(&client, &base, mod_id, "Req B").await;

    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let dash = create_dashboard(&client, &base, ws_id).await;
    let dash_id = dash["id"].as_str().unwrap();

    // Create coverage_chart widget scoped to module
    let res = client
        .post(format!("{}/dashboards/{dash_id}/widgets", api(&base)))
        .json(&json!({
            "widget_type": "coverage_chart",
            "title": "Coverage",
            "config": {"module_ids": [mod_id]}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let widget: Value = res.json().await.unwrap();
    let wid = widget["id"].as_str().unwrap();

    // Get widget data
    let res = client
        .get(format!(
            "{}/dashboards/{dash_id}/widgets/{wid}/data",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let data: Vec<Value> = res.json().await.unwrap();
    // Should return WidgetDataEntry items with label/value
    for entry in &data {
        assert!(entry["label"].is_string());
        assert!(entry["value"].is_number());
    }
}

#[tokio::test]
async fn test_dashboard_widget_data_lifecycle() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let _ = create_object(&client, &base, mod_id, "Lifecycle Obj").await;

    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let dash = create_dashboard(&client, &base, ws_id).await;
    let dash_id = dash["id"].as_str().unwrap();

    let res = client
        .post(format!("{}/dashboards/{dash_id}/widgets", api(&base)))
        .json(&json!({
            "widget_type": "lifecycle_distribution",
            "title": "Lifecycle",
            "config": {"module_ids": [mod_id]}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let widget: Value = res.json().await.unwrap();
    let wid = widget["id"].as_str().unwrap();

    let res = client
        .get(format!(
            "{}/dashboards/{dash_id}/widgets/{wid}/data",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let data: Vec<Value> = res.json().await.unwrap();
    for entry in &data {
        assert!(entry["label"].is_string());
        assert!(entry["value"].is_number());
    }
}

#[tokio::test]
async fn test_dashboard_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!(
            "{}/workspaces/{ws_id}/dashboards/{fake_id}",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_dashboard_widget_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let dash = create_dashboard(&client, &base, ws_id).await;
    let dash_id = dash["id"].as_str().unwrap();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!(
            "{}/dashboards/{dash_id}/widgets/{fake_id}",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
