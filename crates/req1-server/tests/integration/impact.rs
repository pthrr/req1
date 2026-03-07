use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{
    api, authed_client, create_link_type, create_module, create_object, spawn_server,
};

#[tokio::test]
async fn test_impact_analysis() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
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
    let client = authed_client(&base).await;
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
    let client = authed_client(&base).await;
    let fake_id = uuid::Uuid::now_v7();

    let res = client
        .get(format!("{}/object-impact/{fake_id}", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_coverage_metrics() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
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
    let client = authed_client(&base).await;
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
