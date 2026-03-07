use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{
    api, authed_client, create_link_type, create_project, create_two_objects, spawn_server,
};

#[tokio::test]
async fn test_link_type_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let url = format!("{}/link-types", api(&base));

    let res = client
        .post(&url)
        .json(&json!({"name": "satisfies", "description": "Requirement satisfies another"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let lt: Value = res.json().await.unwrap();
    assert_eq!(lt["name"], "satisfies");

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

#[tokio::test]
async fn test_link_crud() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (mod_id, obj1_id, obj2_id) = create_two_objects(&client, &base).await;
    let lt_id = create_link_type(&client, &base).await;
    let url = format!("{}/links", api(&base));

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

    let res = client.get(format!("{url}/{link_id}")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .get(format!("{url}?module_id={mod_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = res.json().await.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    let res = client
        .patch(format!("{url}/{link_id}"))
        .json(&json!({"suspect": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["suspect"], true);

    let res = client
        .patch(format!("{url}/{link_id}"))
        .json(&json!({"suspect": false}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let resolved: Value = res.json().await.unwrap();
    assert_eq!(resolved["suspect"], false);

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
    let client = authed_client(&base).await;
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
    let client = authed_client(&base).await;
    let (_mod_id, obj1_id, obj2_id) = create_two_objects(&client, &base).await;
    let lt_id = create_link_type(&client, &base).await;
    let url = format!("{}/links", api(&base));

    let payload = json!({
        "source_object_id": obj1_id,
        "target_object_id": obj2_id,
        "link_type_id": lt_id,
    });

    let res = client.post(&url).json(&payload).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = client.post(&url).json(&payload).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_suspect_flag_on_object_update() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (mod_id, obj1_id, obj2_id) = create_two_objects(&client, &base).await;
    let lt_id = create_link_type(&client, &base).await;

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

#[tokio::test]
async fn test_traceability_matrix() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (ws, proj) = create_project(&client, &base).await;
    let proj_id = proj["id"].as_str().unwrap();

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

    let _ = ws;
}
