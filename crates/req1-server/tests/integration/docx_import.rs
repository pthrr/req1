use axum::http::StatusCode;
use serde_json::{Value, json};

use super::common::{api, authed_client, build_test_docx, create_module, spawn_server};

#[tokio::test]
async fn test_docx_preview_discovers_styles() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let docx = build_test_docx(&[
        ("Heading1", "Chapter", None),
        ("Normal", "Body text", None),
        ("Heading2", "Sub", None),
    ]);

    let res = client
        .post(format!(
            "{}/modules/{mod_id}/import/docx/preview",
            api(&base)
        ))
        .header("content-type", "application/octet-stream")
        .body(docx)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let preview: Value = res.json().await.unwrap();
    assert_eq!(preview["paragraph_count"], 3);

    let styles = preview["styles"].as_array().unwrap();
    let find_style =
        |name: &str| -> Option<&Value> { styles.iter().find(|s| s["style_id"] == name) };
    let h1 = find_style("Heading1").expect("should discover Heading1");
    assert_eq!(h1["count"], 1);
    let normal = find_style("Normal").expect("should discover Normal");
    assert_eq!(normal["count"], 1);
    let h2 = find_style("Heading2").expect("should discover Heading2");
    assert_eq!(h2["count"], 1);
}

#[tokio::test]
async fn test_docx_import_creates_objects() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let docx = build_test_docx(&[
        ("Heading1", "Req 1", None),
        ("Normal", "Body 1", None),
        ("Heading1", "Req 2", None),
    ]);

    let mapping = json!({
        "style_mappings": [
            {"style_id": "Heading1", "classification": "normative", "is_heading": true},
            {"style_id": "Normal", "classification": "informative", "is_heading": false}
        ]
    });

    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(docx).file_name("test.docx"),
        )
        .part(
            "mapping",
            reqwest::multipart::Part::text(mapping.to_string()),
        );

    let res = client
        .post(format!("{}/modules/{mod_id}/import/docx", api(&base)))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let result: Value = res.json().await.unwrap();
    assert_eq!(result["objects_created"], 2);
    assert_eq!(result["objects_updated"], 0);

    // Verify objects exist
    let res = client
        .get(format!("{}/modules/{mod_id}/objects", api(&base)))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let objs: Value = res.json().await.unwrap();
    let items = objs["items"].as_array().unwrap();
    assert!(items.len() >= 2);
}

#[tokio::test]
async fn test_docx_import_skips_unmapped_styles() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let docx = build_test_docx(&[("Heading1", "Keep", None), ("CustomStyle", "Skip me", None)]);

    let mapping = json!({
        "style_mappings": [
            {"style_id": "Heading1", "classification": "normative", "is_heading": true}
        ]
    });

    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(docx).file_name("test.docx"),
        )
        .part(
            "mapping",
            reqwest::multipart::Part::text(mapping.to_string()),
        );

    let res = client
        .post(format!("{}/modules/{mod_id}/import/docx", api(&base)))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let result: Value = res.json().await.unwrap();
    assert_eq!(result["objects_created"], 1);
    assert_eq!(result["paragraphs_skipped"], 1);
}

#[tokio::test]
async fn test_docx_round_trip_updates_existing() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let mapping = json!({
        "style_mappings": [
            {"style_id": "Heading1", "classification": "normative", "is_heading": true}
        ]
    });

    // First import with bookmark
    let docx1 = build_test_docx(&[("Heading1", "Original", Some("req1_KNOWN_ID"))]);
    let form1 = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(docx1).file_name("test.docx"),
        )
        .part(
            "mapping",
            reqwest::multipart::Part::text(mapping.to_string()),
        );

    let res = client
        .post(format!("{}/modules/{mod_id}/import/docx", api(&base)))
        .multipart(form1)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let r1: Value = res.json().await.unwrap();
    assert_eq!(r1["objects_created"], 1);
    assert_eq!(r1["objects_updated"], 0);

    // Second import with same bookmark but different text
    let docx2 = build_test_docx(&[("Heading1", "Updated", Some("req1_KNOWN_ID"))]);
    let form2 = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(docx2).file_name("test.docx"),
        )
        .part(
            "mapping",
            reqwest::multipart::Part::text(mapping.to_string()),
        );

    let res = client
        .post(format!("{}/modules/{mod_id}/import/docx", api(&base)))
        .multipart(form2)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let r2: Value = res.json().await.unwrap();
    assert_eq!(r2["objects_updated"], 1);
    assert_eq!(r2["objects_created"], 0);

    // Verify the heading was updated
    let res = client
        .get(format!("{}/modules/{mod_id}/objects", api(&base)))
        .send()
        .await
        .unwrap();
    let objs: Value = res.json().await.unwrap();
    let items = objs["items"].as_array().unwrap();
    let updated_obj = items
        .iter()
        .find(|o| o["docx_source_id"] == "KNOWN_ID")
        .expect("should find object with docx_source_id");
    assert_eq!(updated_obj["heading"], "Updated");
}
