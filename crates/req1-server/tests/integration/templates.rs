use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, create_workspace, spawn_server};

#[tokio::test]
async fn test_project_template_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let url = format!("{}/project-templates", api(&base));

    // Create
    let res = client
        .post(&url)
        .json(&json!({
            "name": "Test Template",
            "description": "A test",
            "standard": "TEST-001",
            "template_data": {
                "modules": [{
                    "name": "Mod1",
                    "prefix": "TST",
                    "separator": "-",
                    "digits": 3
                }]
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let tmpl: Value = res.json().await.unwrap();
    let tmpl_id = tmpl["id"].as_str().unwrap();
    assert_eq!(tmpl["name"], "Test Template");
    assert_eq!(tmpl["is_builtin"], false);

    // Get
    let res = client.get(format!("{url}/{tmpl_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got: Value = res.json().await.unwrap();
    assert_eq!(got["standard"], "TEST-001");

    // Update
    let res = client
        .patch(format!("{url}/{tmpl_id}"))
        .json(&json!({"name": "Renamed"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed");

    // List
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Vec<Value> = res.json().await.unwrap();
    assert!(list.iter().any(|i| i["id"] == tmpl_id));

    // Delete
    let res = client
        .delete(format!("{url}/{tmpl_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{tmpl_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_builtin_templates_exist() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;

    let res = client
        .get(format!("{}/project-templates", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Vec<Value> = res.json().await.unwrap();

    let builtins: Vec<&Value> = list.iter().filter(|t| t["is_builtin"] == true).collect();
    assert!(
        builtins.len() >= 3,
        "expected at least 3 built-in templates, got {}",
        builtins.len()
    );

    let names: Vec<&str> = builtins.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.iter().any(|n| n.contains("ISO 26262")));
    assert!(names.iter().any(|n| n.contains("DO-178C")));
    assert!(names.iter().any(|n| n.contains("IEC 62304")));
}

#[tokio::test]
async fn test_builtin_template_cannot_be_deleted() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;

    let res = client
        .get(format!("{}/project-templates", api(&base)))
        .send()
        .await
        .unwrap();
    let list: Vec<Value> = res.json().await.unwrap();

    let builtin = list
        .iter()
        .find(|t| t["is_builtin"] == true)
        .expect("should have at least one builtin");
    let builtin_id = builtin["id"].as_str().unwrap();

    let res = client
        .delete(format!("{}/project-templates/{builtin_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_template_instantiate_creates_project() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();

    // Get a builtin template
    let res = client
        .get(format!("{}/project-templates", api(&base)))
        .send()
        .await
        .unwrap();
    let list: Vec<Value> = res.json().await.unwrap();
    let builtin = list
        .iter()
        .find(|t| t["is_builtin"] == true)
        .expect("should have builtins");
    let tmpl_id = builtin["id"].as_str().unwrap();

    // Instantiate
    let res = client
        .post(format!(
            "{}/project-templates/{tmpl_id}/instantiate",
            api(&base)
        ))
        .json(&json!({
            "workspace_id": ws_id,
            "project_name": "My ISO Project",
            "include_seed_objects": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let result: Value = res.json().await.unwrap();
    assert!(result["project_id"].is_string());
    let modules_created = result["modules_created"].as_u64().unwrap();
    assert!(modules_created > 0, "should create at least one module");

    // Verify project exists
    let proj_id = result["project_id"].as_str().unwrap();
    let res = client
        .get(format!(
            "{}/workspaces/{ws_id}/projects/{proj_id}",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let proj: Value = res.json().await.unwrap();
    assert_eq!(proj["name"], "My ISO Project");
}

#[tokio::test]
async fn test_template_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/project-templates/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = client
        .post(format!(
            "{}/project-templates/{fake_id}/instantiate",
            api(&base)
        ))
        .json(&json!({
            "workspace_id": fake_id,
            "project_name": "No Template"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
