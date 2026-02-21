//! Integration tests for the req1-server API.
//!
//! These tests spin up a real Axum server backed by a PostgreSQL test database.
//! Requires a running PostgreSQL instance — uses `TEST_DATABASE_URL` env var
//! (falls back to `DATABASE_URL`).
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::doc_markdown,
    clippy::let_underscore_future
)]

use axum::http::StatusCode;
use reqwest::Client;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use serde_json::{Value, json};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use req1_server::config::Config;
use req1_server::routes;
use req1_server::state::AppState;

/// Spin up a test server on a random port and return its base URL.
async fn spawn_server() -> String {
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be set");

    let db = Database::connect(&database_url)
        .await
        .expect("failed to connect to test database");

    // Run migrations
    migration::Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");

    let config = Config {
        database_url,
        redis_url: None,
        listen_addr: "127.0.0.1:0".to_string(),
    };

    let state = AppState { db, config };

    let app = routes::router()
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind");
    let addr: SocketAddr = listener.local_addr().expect("failed to get local addr");

    let _ = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    format!("http://{addr}")
}

fn api(base: &str) -> String {
    format!("{base}/api/v1")
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_liveness() {
    let base = spawn_server().await;
    let client = Client::new();

    let res = client
        .get(format!("{base}/health/live"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_health_readiness() {
    let base = spawn_server().await;
    let client = Client::new();

    let res = client
        .get(format!("{base}/health/ready"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

// ---------------------------------------------------------------------------
// Workspaces CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_workspace_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let url = format!("{}/workspaces", api(&base));

    // Create
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

    // Get
    let res = client.get(format!("{url}/{ws_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got: Value = res.json().await.unwrap();
    assert_eq!(got["id"], ws_id);

    // List (use high limit to account for other tests' data)
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

    // Update
    let res = client
        .patch(format!("{url}/{ws_id}"))
        .json(&json!({"name": "Renamed Workspace"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed Workspace");

    // Delete
    let res = client
        .delete(format!("{url}/{ws_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{ws_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_workspace_get_not_found() {
    let base = spawn_server().await;
    let client = Client::new();
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
    let client = Client::new();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .delete(format!("{}/workspaces/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Projects CRUD
// ---------------------------------------------------------------------------

async fn create_workspace(client: &Client, base: &str) -> Value {
    client
        .post(format!("{}/workspaces", api(base)))
        .json(&json!({"name": format!("ws-{}", uuid::Uuid::now_v7())}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn test_project_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let url = format!("{}/workspaces/{ws_id}/projects", api(&base));

    // Create
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

    // Get
    let res = client.get(format!("{url}/{proj_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
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

    // Update
    let res = client
        .patch(format!("{url}/{proj_id}"))
        .json(&json!({"name": "Renamed Project"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Renamed Project");

    // Delete
    let res = client
        .delete(format!("{url}/{proj_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{proj_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Modules CRUD
// ---------------------------------------------------------------------------

async fn create_project(client: &Client, base: &str) -> (Value, Value) {
    let ws = create_workspace(client, base).await;
    let ws_id = ws["id"].as_str().unwrap();
    let proj: Value = client
        .post(format!("{}/workspaces/{ws_id}/projects", api(base)))
        .json(&json!({"name": format!("proj-{}", uuid::Uuid::now_v7())}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    (ws, proj)
}

#[tokio::test]
async fn test_module_crud() {
    let base = spawn_server().await;
    let client = Client::new();
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

// ---------------------------------------------------------------------------
// Objects CRUD + History
// ---------------------------------------------------------------------------

async fn create_module(client: &Client, base: &str) -> (Value, Value, Value) {
    let (ws, proj) = create_project(client, base).await;
    let proj_id = proj["id"].as_str().unwrap();
    let module: Value = client
        .post(format!("{}/modules", api(base)))
        .json(&json!({"name": format!("mod-{}", uuid::Uuid::now_v7()), "project_id": proj_id}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    (ws, proj, module)
}

#[tokio::test]
async fn test_object_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create
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

    // Get
    let res = client.get(format!("{url}/{obj_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    // Update (version should increment)
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

    // History — should have 2 entries (create + update)
    let res = client
        .get(format!("{url}/{obj_id}/history"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let history: Value = res.json().await.unwrap();
    assert_eq!(history["items"].as_array().unwrap().len(), 2);

    // Delete
    let res = client
        .delete(format!("{url}/{obj_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // History still preserved after delete (migration 0004)
    let res = client
        .get(format!("{url}/{obj_id}/history"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let history_after: Value = res.json().await.unwrap();
    // Should have 3 entries: create + update + delete
    assert_eq!(history_after["items"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_object_filters() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create two objects
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

    // Filter by heading
    let res = client
        .get(format!("{url}?heading=ALPHA"))
        .send()
        .await
        .unwrap();
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["heading"], "REQ-ALPHA");

    // Filter by body
    let res = client.get(format!("{url}?body=Beta")).send().await.unwrap();
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    // Sort by heading desc
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

// ---------------------------------------------------------------------------
// Link Types + Links
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_link_type_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let url = format!("{}/link-types", api(&base));

    // Create
    let res = client
        .post(&url)
        .json(&json!({"name": "satisfies", "description": "Requirement satisfies another"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let lt: Value = res.json().await.unwrap();
    assert_eq!(lt["name"], "satisfies");

    // List
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .any(|i| i["name"] == "satisfies")
    );
}

async fn create_two_objects(client: &Client, base: &str) -> (String, String, String) {
    let (_ws, _proj, module) = create_module(client, base).await;
    let mod_id = module["id"].as_str().unwrap().to_string();
    let url = format!("{}/modules/{mod_id}/objects", api(base));

    let obj1: Value = client
        .post(&url)
        .json(&json!({"heading": "REQ-SRC"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let obj2: Value = client
        .post(&url)
        .json(&json!({"heading": "REQ-TGT"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    (
        mod_id,
        obj1["id"].as_str().unwrap().to_string(),
        obj2["id"].as_str().unwrap().to_string(),
    )
}

async fn create_link_type(client: &Client, base: &str) -> String {
    let lt: Value = client
        .post(format!("{}/link-types", api(base)))
        .json(&json!({"name": format!("lt-{}", uuid::Uuid::now_v7())}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    lt["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_link_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (mod_id, obj1_id, obj2_id) = create_two_objects(&client, &base).await;
    let lt_id = create_link_type(&client, &base).await;
    let url = format!("{}/links", api(&base));

    // Create link
    let res = client
        .post(&url)
        .json(&json!({
            "source_object_id": obj1_id,
            "target_object_id": obj2_id,
            "link_type_id": lt_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let link: Value = res.json().await.unwrap();
    let link_id = link["id"].as_str().unwrap();
    assert_eq!(link["suspect"], false);

    // Get
    let res = client.get(format!("{url}/{link_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List by module
    let res = client
        .get(format!("{url}?module_id={mod_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    // Update suspect
    let res = client
        .patch(format!("{url}/{link_id}"))
        .json(&json!({"suspect": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["suspect"], true);

    // Resolve suspect
    let res = client
        .patch(format!("{url}/{link_id}"))
        .json(&json!({"suspect": false}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let resolved: Value = res.json().await.unwrap();
    assert_eq!(resolved["suspect"], false);

    // Delete
    let res = client
        .delete(format!("{url}/{link_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_link_self_reference_rejected() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_mod_id, obj1_id, _obj2_id) = create_two_objects(&client, &base).await;
    let lt_id = create_link_type(&client, &base).await;

    let res = client
        .post(format!("{}/links", api(&base)))
        .json(&json!({
            "source_object_id": obj1_id,
            "target_object_id": obj1_id,
            "link_type_id": lt_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_link_duplicate_rejected() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_mod_id, obj1_id, obj2_id) = create_two_objects(&client, &base).await;
    let lt_id = create_link_type(&client, &base).await;
    let url = format!("{}/links", api(&base));

    let payload = json!({
        "source_object_id": obj1_id,
        "target_object_id": obj2_id,
        "link_type_id": lt_id,
    });

    // First create succeeds
    let res = client.post(&url).json(&payload).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Duplicate rejected
    let res = client.post(&url).json(&payload).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_suspect_flag_on_object_update() {
    let base = spawn_server().await;
    let client = Client::new();
    let (mod_id, obj1_id, obj2_id) = create_two_objects(&client, &base).await;
    let lt_id = create_link_type(&client, &base).await;

    // Create link (not suspect)
    let link: Value = client
        .post(format!("{}/links", api(&base)))
        .json(&json!({
            "source_object_id": obj1_id,
            "target_object_id": obj2_id,
            "link_type_id": lt_id,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let link_id = link["id"].as_str().unwrap();

    // Update source object → link should become suspect
    let _ = client
        .patch(format!("{}/modules/{mod_id}/objects/{obj1_id}", api(&base)))
        .json(&json!({"heading": "REQ-SRC-v2"}))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{}/links/{link_id}", api(&base)))
        .send()
        .await
        .unwrap();
    let updated_link: Value = res.json().await.unwrap();
    assert_eq!(updated_link["suspect"], true);
}

// ---------------------------------------------------------------------------
// Baselines
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_baseline_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let obj_url = format!("{}/modules/{mod_id}/objects", api(&base));
    let bl_url = format!("{}/modules/{mod_id}/baselines", api(&base));

    // Create an object first
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

    // Create baseline (snapshots current state)
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

    // Get baseline
    let res = client
        .get(format!("{bl_url}/{bl_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List baselines
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

    // Delete baseline
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
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let obj_url = format!("{}/modules/{mod_id}/objects", api(&base));
    let bl_url = format!("{}/modules/{mod_id}/baselines", api(&base));

    // Create object
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

    // Baseline A (v1)
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

    // Update object to v2
    let _ = client
        .patch(format!("{obj_url}/{obj_id}"))
        .json(&json!({"heading": "REQ-DIFF-v2"}))
        .send()
        .await
        .unwrap();

    // Add a new object
    let _ = client
        .post(&obj_url)
        .json(&json!({"heading": "REQ-NEW"}))
        .send()
        .await
        .unwrap();

    // Baseline B (v2 + new object)
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

    // Diff
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

    // One added (the new object), one modified (REQ-DIFF v1→v2)
    assert_eq!(diff["added"].as_array().unwrap().len(), 1);
    assert_eq!(diff["modified"].as_array().unwrap().len(), 1);
    assert_eq!(diff["removed"].as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// Attribute Definitions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_attribute_definition_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    // Create string attr
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

    // Get
    let res = client.get(format!("{url}/{attr_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
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

    // Update
    let res = client
        .patch(format!("{url}/{attr_id}"))
        .json(&json!({"name": "importance"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "importance");

    // Delete
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
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    // Create enum attr with valid values
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
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    // Enum without enum_values → rejected
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
    let client = Client::new();
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
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    // integer with non-int default
    let res = client
        .post(&url)
        .json(&json!({"name": "count", "data_type": "integer", "default_value": "abc"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // bool with bad default
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
    let client = Client::new();
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

// ---------------------------------------------------------------------------
// Object Attribute Validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_object_attribute_validation() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let attr_url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));
    let obj_url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create an enum attr definition
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

    // Create an integer attr definition
    let _ = client
        .post(&attr_url)
        .json(&json!({"name": "weight", "data_type": "integer"}))
        .send()
        .await
        .unwrap();

    // Valid attributes
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

    // Invalid enum value
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

    // Invalid type for integer
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

    // Unknown attribute
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

// ---------------------------------------------------------------------------
// Traceability Matrix
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_traceability_matrix() {
    let base = spawn_server().await;
    let client = Client::new();
    let (ws, proj) = create_project(&client, &base).await;
    let proj_id = proj["id"].as_str().unwrap();

    // Create two modules
    let mod1: Value = client
        .post(format!("{}/modules", api(&base)))
        .json(&json!({"name": "Source Module", "project_id": proj_id}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mod2: Value = client
        .post(format!("{}/modules", api(&base)))
        .json(&json!({"name": "Target Module", "project_id": proj_id}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let mod1_id = mod1["id"].as_str().unwrap();
    let mod2_id = mod2["id"].as_str().unwrap();

    // Create objects in each module
    let src_obj: Value = client
        .post(format!("{}/modules/{mod1_id}/objects", api(&base)))
        .json(&json!({"heading": "SRC-001"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tgt_obj: Value = client
        .post(format!("{}/modules/{mod2_id}/objects", api(&base)))
        .json(&json!({"heading": "TGT-001"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Create link type and link
    let lt_id = create_link_type(&client, &base).await;
    let _ = client
        .post(format!("{}/links", api(&base)))
        .json(&json!({
            "source_object_id": src_obj["id"],
            "target_object_id": tgt_obj["id"],
            "link_type_id": lt_id,
        }))
        .send()
        .await
        .unwrap();

    // Get matrix
    let res = client
        .get(format!(
            "{}/traceability-matrix?source_module_id={mod1_id}&target_module_id={mod2_id}",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let matrix: Value = res.json().await.unwrap();
    assert_eq!(matrix["source_objects"].as_array().unwrap().len(), 1);
    assert_eq!(matrix["target_objects"].as_array().unwrap().len(), 1);
    assert_eq!(matrix["cells"].as_array().unwrap().len(), 1);
    assert_eq!(matrix["cells"][0]["suspect"], false);

    // Suppress unused variable warning
    let _ = ws;
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_pagination() {
    let base = spawn_server().await;
    let client = Client::new();
    let url = format!("{}/workspaces", api(&base));

    // Create 5 workspaces
    for i in 0..5 {
        let _ = client
            .post(&url)
            .json(&json!({"name": format!("paginated-ws-{i}")}))
            .send()
            .await
            .unwrap();
    }

    // Page size 2
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

// ---------------------------------------------------------------------------
// Cascade Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_workspace_cascade_deletes_project() {
    let base = spawn_server().await;
    let client = Client::new();
    let ws = create_workspace(&client, &base).await;
    let ws_id = ws["id"].as_str().unwrap();

    // Create a project
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

    // Delete workspace
    let res = client
        .delete(format!("{}/workspaces/{ws_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Project should be gone
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

#[tokio::test]
async fn test_enum_values_non_array_rejected() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/attribute-definitions", api(&base));

    // enum_values on non-enum type should be rejected
    let res = client
        .post(&url)
        .json(&json!({"name": "x", "data_type": "string", "enum_values": ["a", "b"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Helper: create a single object (used by tests that need standalone objects)
// ---------------------------------------------------------------------------

async fn create_object(client: &Client, base: &str, mod_id: &str, heading: &str) -> Value {
    client
        .post(format!("{}/modules/{mod_id}/objects", api(base)))
        .json(&json!({"heading": heading}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// Views CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_view_crud() {
    let base = spawn_server().await;
    let client = Client::new();
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
    let client = Client::new();
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

// ---------------------------------------------------------------------------
// Object Types CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_object_type_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/object-types", api(&base));

    // Create
    let res = client
        .post(&url)
        .json(&json!({
            "module_id": mod_id,
            "name": "Functional Requirement",
            "required_attributes": ["priority"],
            "attribute_schema": {"weight": {"type": "integer"}},
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let ot: Value = res.json().await.unwrap();
    let ot_id = ot["id"].as_str().unwrap();
    assert_eq!(ot["name"], "Functional Requirement");

    // Get
    let res = client.get(format!("{url}/{ot_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
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

    // Update
    let res = client
        .patch(format!("{url}/{ot_id}"))
        .json(&json!({"name": "Non-Functional Requirement"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Non-Functional Requirement");

    // Delete
    let res = client
        .delete(format!("{url}/{ot_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{ot_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_object_type_not_found() {
    let base = spawn_server().await;
    let client = Client::new();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/object-types/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Comments CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_comment_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let obj = create_object(&client, &base, mod_id, "REQ-COMMENT").await;
    let obj_id = obj["id"].as_str().unwrap();
    let url = format!("{}/objects/{obj_id}/comments", api(&base));

    // Create
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

    // Get
    let res = client
        .get(format!("{url}/{comment_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    // Update body
    let res = client
        .patch(format!("{url}/{comment_id}"))
        .json(&json!({"body": "Updated comment"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["body"], "Updated comment");

    // Resolve
    let res = client
        .patch(format!("{url}/{comment_id}"))
        .json(&json!({"resolved": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let resolved: Value = res.json().await.unwrap();
    assert_eq!(resolved["resolved"], true);

    // Delete
    let res = client
        .delete(format!("{url}/{comment_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
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
    let client = Client::new();
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

// ---------------------------------------------------------------------------
// App Users CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_app_user_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let url = format!("{}/users", api(&base));

    // Create
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

    // Get
    let res = client.get(format!("{url}/{user_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
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

    // Update
    let res = client
        .patch(format!("{url}/{user_id}"))
        .json(&json!({"display_name": "Updated User"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["display_name"], "Updated User");

    // Delete
    let res = client
        .delete(format!("{url}/{user_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{user_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_app_user_invalid_role() {
    let base = spawn_server().await;
    let client = Client::new();

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
    let client = Client::new();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/users/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Review Packages CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_review_package_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/review-packages", api(&base));

    // Create
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

    // Get
    let res = client.get(format!("{url}/{rp_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
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

    // Update name
    let res = client
        .patch(format!("{url}/{rp_id}"))
        .json(&json!({"name": "Updated Package"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Updated Package");

    // Delete
    let res = client
        .delete(format!("{url}/{rp_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{rp_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_review_package_not_found() {
    let base = spawn_server().await;
    let client = Client::new();
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

// ---------------------------------------------------------------------------
// Review Assignments CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_review_assignment_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    // Create review package first
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

    // Create assignment
    let res = client.post(&url).json(&json!({})).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let ra: Value = res.json().await.unwrap();
    let ra_id = ra["id"].as_str().unwrap();
    assert_eq!(ra["status"], "pending");

    // Get
    let res = client.get(format!("{url}/{ra_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    // Update status
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

    // Delete
    let res = client
        .delete(format!("{url}/{ra_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// Change Proposals CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_change_proposal_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/change-proposals", api(&base));

    // Create
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

    // Get
    let res = client.get(format!("{url}/{cp_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
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

    // Update title + status
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

    // Delete
    let res = client
        .delete(format!("{url}/{cp_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{cp_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_change_proposal_invalid_status() {
    let base = spawn_server().await;
    let client = Client::new();
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

    // Update with invalid status
    let res = client
        .patch(format!("{url}/{cp_id}"))
        .json(&json!({"status": "bogus"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Baseline Sets CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_baseline_set_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let url = format!("{}/baseline-sets", api(&base));

    // Create
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

    // Get
    let res = client.get(format!("{url}/{bs_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
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

    // Update
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

    // Delete
    let res = client
        .delete(format!("{url}/{bs_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{bs_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Scripts CRUD + Test + Execute
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_script_crud() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/scripts", api(&base));

    // Create trigger script
    let res = client
        .post(&url)
        .json(&json!({
            "name": "Auto-classify",
            "script_type": "trigger",
            "hook_point": "pre_save",
            "source_code": "return { rejected = false }",
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

    // Get
    let res = client.get(format!("{url}/{s_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // List
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert!(list.as_array().unwrap().iter().any(|i| i["id"] == s_id));

    // Update source_code
    let res = client
        .patch(format!("{url}/{s_id}"))
        .json(&json!({"source_code": "return { rejected = false, reason = 'ok' }"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Delete
    let res = client.delete(format!("{url}/{s_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let res = client.get(format!("{url}/{s_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_script_invalid_type() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let res = client
        .post(format!("{}/modules/{mod_id}/scripts", api(&base)))
        .json(&json!({
            "name": "Bad Script",
            "script_type": "invalid",
            "source_code": "return nil",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_script_trigger_requires_hook_point() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let res = client
        .post(format!("{}/modules/{mod_id}/scripts", api(&base)))
        .json(&json!({
            "name": "No Hook",
            "script_type": "trigger",
            "source_code": "return { rejected = false }",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_script_test_endpoint() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/scripts", api(&base));

    // Create a layout script that returns the heading
    let s: Value = client
        .post(&url)
        .json(&json!({
            "name": "Echo heading",
            "script_type": "layout",
            "source_code": "return obj.heading or ''",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let s_id = s["id"].as_str().unwrap();

    // Test with a mock object
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
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/scripts", api(&base));

    // Create an action script that outputs a message
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

    // Execute
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
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    // Create objects
    let _ = create_object(&client, &base, mod_id, "OBJ-A").await;
    let _ = create_object(&client, &base, mod_id, "OBJ-B").await;

    // Create layout script
    let script_url = format!("{}/modules/{mod_id}/scripts", api(&base));
    let s: Value = client
        .post(&script_url)
        .json(&json!({
            "name": "Level layout",
            "script_type": "layout",
            "source_code": "return obj.heading or 'none'",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let s_id = s["id"].as_str().unwrap();

    // Batch layout
    let res = client
        .post(format!("{script_url}/{s_id}/layout"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let result: Value = res.json().await.unwrap();
    assert_eq!(result["results"].as_array().unwrap().len(), 2);
}

// ---------------------------------------------------------------------------
// Impact Analysis
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_impact_analysis() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    // Create 3 objects in a chain
    let obj1 = create_object(&client, &base, mod_id, "IMPACT-A").await;
    let obj2 = create_object(&client, &base, mod_id, "IMPACT-B").await;
    let obj3 = create_object(&client, &base, mod_id, "IMPACT-C").await;
    let obj1_id = obj1["id"].as_str().unwrap();
    let obj2_id = obj2["id"].as_str().unwrap();
    let obj3_id = obj3["id"].as_str().unwrap();

    let lt_id = create_link_type(&client, &base).await;

    // A -> B -> C
    let _ = client
        .post(format!("{}/links", api(&base)))
        .json(&json!({
            "source_object_id": obj1_id,
            "target_object_id": obj2_id,
            "link_type_id": lt_id,
        }))
        .send()
        .await
        .unwrap();
    let _ = client
        .post(format!("{}/links", api(&base)))
        .json(&json!({
            "source_object_id": obj2_id,
            "target_object_id": obj3_id,
            "link_type_id": lt_id,
        }))
        .send()
        .await
        .unwrap();

    // Forward impact from A
    let res = client
        .get(format!(
            "{}/object-impact/{obj1_id}?direction=forward",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let impact: Value = res.json().await.unwrap();
    assert_eq!(impact["root_id"], obj1_id);
    assert_eq!(impact["direction"], "forward");
    // Should find B (depth 1) and C (depth 2)
    let objects = impact["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 2);
    let edges = impact["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 2);
}

#[tokio::test]
async fn test_impact_invalid_direction() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let obj = create_object(&client, &base, mod_id, "IMPACT-DIR").await;
    let obj_id = obj["id"].as_str().unwrap();

    let res = client
        .get(format!(
            "{}/object-impact/{obj_id}?direction=sideways",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_impact_not_found() {
    let base = spawn_server().await;
    let client = Client::new();
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/object-impact/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Coverage Endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_coverage_metrics() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    // Create 2 objects
    let obj1 = create_object(&client, &base, mod_id, "COV-A").await;
    let obj2 = create_object(&client, &base, mod_id, "COV-B").await;
    let obj1_id = obj1["id"].as_str().unwrap();
    let obj2_id = obj2["id"].as_str().unwrap();

    // Link A -> B
    let lt_id = create_link_type(&client, &base).await;
    let _ = client
        .post(format!("{}/links", api(&base)))
        .json(&json!({
            "source_object_id": obj1_id,
            "target_object_id": obj2_id,
            "link_type_id": lt_id,
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{}/modules/{mod_id}/coverage", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let cov: Value = res.json().await.unwrap();
    assert_eq!(cov["total_objects"], 2);
    assert!(cov["with_any_link"].as_u64().unwrap() >= 1);
    assert!(cov["any_link_pct"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_coverage_empty_module() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let res = client
        .get(format!("{}/modules/{mod_id}/coverage", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let cov: Value = res.json().await.unwrap();
    assert_eq!(cov["total_objects"], 0);
    assert_eq!(cov["upstream_pct"], 0.0);
    assert_eq!(cov["downstream_pct"], 0.0);
    assert_eq!(cov["any_link_pct"], 0.0);
}

// ---------------------------------------------------------------------------
// Module Templates
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_module_from_template() {
    let base = spawn_server().await;
    let client = Client::new();
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
            "source_code": "return { value = '' }",
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
    let client = Client::new();
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

// ---------------------------------------------------------------------------
// Validation Endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_validate_module() {
    let base = spawn_server().await;
    let client = Client::new();
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

// ---------------------------------------------------------------------------
// Publish Endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_publish_html() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    // Create an object so there's content
    let _ = create_object(&client, &base, mod_id, "PUB-001").await;

    let res = client
        .get(format!("{}/modules/{mod_id}/publish", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.text().await.unwrap();
    assert!(body.contains("html") || body.contains("HTML"));
}

#[tokio::test]
async fn test_publish_unsupported_format() {
    let base = spawn_server().await;
    let client = Client::new();
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=pdf",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
