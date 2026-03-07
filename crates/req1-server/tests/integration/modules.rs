use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{
    api, authed_client, create_module, create_object, create_project, spawn_server,
};

#[tokio::test]
async fn test_module_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, proj) = create_project(&client, &base).await;
    let proj_id = proj["id"].as_str().unwrap();
    let url = format!("{}/modules", api(&base));

    // Create
    let res = client
        .post(&url)
        .json(&json!({"name": "Test Module", "project_id": proj_id}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let module: Value = res.json().await.unwrap();
    let mod_id = module["id"].as_str().unwrap();
    assert_eq!(module["name"], "Test Module");

    // Get
    let res = client.get(format!("{url}/{mod_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List with filter
    let res = client
        .get(format!("{url}?project_id={proj_id}"))
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
            .any(|i| i["id"] == mod_id)
    );

    // Update
    let res = client
        .patch(format!("{url}/{mod_id}"))
        .json(&json!({"name": "Renamed Module"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed Module");

    // Delete
    let res = client
        .delete(format!("{url}/{mod_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_view_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/views", api(&base));

    // Create
    let res = client
        .post(&url)
        .json(&json!({
            "name": "My View",
            "column_config": {"columns": ["heading", "body"]},
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let view: Value = res.json().await.unwrap();
    let view_id = view["id"].as_str().unwrap();
    assert_eq!(view["name"], "My View");

    // Get
    let res = client.get(format!("{url}/{view_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got: Value = res.json().await.unwrap();
    assert_eq!(got["id"], view_id);

    // List
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == view_id)
    );

    // Update
    let res = client
        .patch(format!("{url}/{view_id}"))
        .json(&json!({"name": "Renamed View"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed View");

    // Delete
    let res = client
        .delete(format!("{url}/{view_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{view_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_view_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/modules/{mod_id}/views/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_module_from_template() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, proj, module) = create_module(&client, &base).await;
    let proj_id = proj["id"].as_str().unwrap();
    let mod_id = module["id"].as_str().unwrap();

    // Add an attribute definition to the template module
    let _ = client
        .post(format!(
            "{}/modules/{mod_id}/attribute-definitions",
            api(&base)
        ))
        .json(&json!({"name": "priority", "data_type": "string"}))
        .send()
        .await
        .unwrap();

    // Add a script to the template module
    let _ = client
        .post(format!("{}/modules/{mod_id}/scripts", api(&base)))
        .json(&json!({
            "name": "Template Script",
            "script_type": "layout",
            "source_code": "return ''",
        }))
        .send()
        .await
        .unwrap();

    // Create module from template
    let res = client
        .post(format!("{}/modules/from-template", api(&base)))
        .json(&json!({
            "name": "From Template",
            "project_id": proj_id,
            "template_module_id": mod_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let new_mod: Value = res.json().await.unwrap();
    let new_mod_id = new_mod["id"].as_str().unwrap();
    assert_eq!(new_mod["name"], "From Template");

    // Verify attr definitions were copied
    let res = client
        .get(format!(
            "{}/modules/{new_mod_id}/attribute-definitions",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    let attrs: Value = res.json().await.unwrap();
    assert!(
        attrs["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["name"] == "priority")
    );

    // Verify scripts were copied
    let res = client
        .get(format!("{}/modules/{new_mod_id}/scripts", api(&base)))
        .send()
        .await
        .unwrap();
    let scripts: Value = res.json().await.unwrap();
    assert!(
        scripts
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == "Template Script")
    );
}

#[tokio::test]
async fn test_module_from_template_not_found() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, proj, _module) = create_module(&client, &base).await;
    let proj_id = proj["id"].as_str().unwrap();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .post(format!("{}/modules/from-template", api(&base)))
        .json(&json!({
            "name": "Bad Template",
            "project_id": proj_id,
            "template_module_id": fake_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_validate_module() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    // Create some objects
    let _ = create_object(&client, &base, mod_id, "VAL-001").await;
    let _ = create_object(&client, &base, mod_id, "VAL-002").await;

    let res = client
        .get(format!("{}/modules/{mod_id}/validate", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let report: Value = res.json().await.unwrap();
    assert!(report["issues"].is_array());
    assert!(report["object_count"].as_u64().unwrap() >= 2);
}
