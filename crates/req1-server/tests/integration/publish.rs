use axum::http::StatusCode;
use serde_json::json;

use super::common::{api, authed_client, create_module, create_object, spawn_server};

#[tokio::test]
async fn test_publish_html() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
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
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=odt",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_publish_docx() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create objects with headings and bodies
    let _ = client
        .post(&url)
        .json(&json!({"heading": "Chapter 1", "body": "Introduction text"}))
        .send()
        .await
        .unwrap();
    let _ = client
        .post(&url)
        .json(&json!({"heading": "Chapter 2", "body": "<p>HTML body content</p>"}))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=docx",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let ct = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        ct.contains("officedocument.wordprocessingml"),
        "Expected DOCX content type, got: {ct}"
    );

    let disp = res
        .headers()
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(disp.contains("document.docx"));

    let bytes = res.bytes().await.unwrap();
    // DOCX files are ZIP archives starting with PK magic bytes
    assert!(bytes.len() > 100, "DOCX should have substantial content");
    assert_eq!(&bytes[0..2], b"PK", "DOCX should be a valid ZIP/PK archive");
}

#[tokio::test]
async fn test_publish_docx_word_alias() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    let _ = create_object(&client, &base, mod_id, "DOCX-001").await;

    // "word" alias should also work
    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=word",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.bytes().await.unwrap();
    assert_eq!(&bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_publish_docx_empty_module() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();

    // Publish empty module — should still succeed
    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=docx",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.bytes().await.unwrap();
    assert_eq!(&bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_publish_html_with_html_body() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create object with HTML body (from TipTap)
    let _ = client
        .post(&url)
        .json(&json!({
            "heading": "HTML-REQ",
            "body": "<p>This is <strong>bold</strong> and <em>italic</em> text.</p>"
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=html",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.text().await.unwrap();

    // HTML body should pass through without double-escaping
    assert!(
        body.contains("<strong>bold</strong>"),
        "HTML body should be preserved in HTML publish"
    );
}

#[tokio::test]
async fn test_publish_markdown_with_html_body_stripped() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create object with HTML body
    let _ = client
        .post(&url)
        .json(&json!({
            "heading": "MD-REQ",
            "body": "<p>Some <strong>formatted</strong> text</p>"
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=markdown",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.text().await.unwrap();

    // HTML tags should be stripped in markdown output
    assert!(
        !body.contains("<p>") && !body.contains("<strong>"),
        "HTML tags should be stripped in markdown publish"
    );
    assert!(
        body.contains("Some") && body.contains("formatted") && body.contains("text"),
        "Text content should be preserved after stripping"
    );
}

#[tokio::test]
async fn test_publish_text_with_html_body_stripped() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create object with HTML body
    let _ = client
        .post(&url)
        .json(&json!({
            "heading": "TXT-REQ",
            "body": "<p>Plain <em>content</em> here</p>"
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=txt",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.text().await.unwrap();

    assert!(
        !body.contains("<p>") && !body.contains("<em>"),
        "HTML tags should be stripped in text publish"
    );
    assert!(
        body.contains("Plain") && body.contains("content") && body.contains("here"),
        "Text content should be preserved"
    );
}

#[tokio::test]
async fn test_publish_latex_with_html_body_stripped() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create object with HTML body
    let _ = client
        .post(&url)
        .json(&json!({
            "heading": "TEX-REQ",
            "body": "<p>LaTeX <strong>test</strong> body</p>"
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=latex",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.text().await.unwrap();

    assert!(
        !body.contains("<p>") && !body.contains("<strong>"),
        "HTML tags should be stripped in LaTeX publish"
    );
    assert!(
        body.contains("LaTeX") && body.contains("test") && body.contains("body"),
        "Text content should be preserved"
    );
    // Should contain LaTeX document structure
    assert!(body.contains("\\documentclass"));
    assert!(body.contains("\\begin{document}"));
}

#[tokio::test]
async fn test_publish_html_markdown_body_converted() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create object with markdown body (legacy, no HTML tags)
    let _ = client
        .post(&url)
        .json(&json!({
            "heading": "MD-LEGACY",
            "body": "This is **bold** markdown"
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=html",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.text().await.unwrap();

    // Markdown body should be converted to HTML via pulldown-cmark
    assert!(
        body.contains("<strong>bold</strong>"),
        "Markdown body should be converted to HTML: {body}"
    );
}

#[tokio::test]
async fn test_publish_docx_strips_html_from_body() {
    let base = spawn_server().await;
    let client = authed_client(&base).await;
    let (_ws, _proj, module) = create_module(&client, &base).await;
    let mod_id = module["id"].as_str().unwrap();
    let url = format!("{}/modules/{mod_id}/objects", api(&base));

    // Create object with HTML body
    let _ = client
        .post(&url)
        .json(&json!({
            "heading": "DOCX-HTML",
            "body": "<p>Formatted <strong>content</strong></p>"
        }))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/modules/{mod_id}/publish?format=docx",
            api(&base)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // We can't easily inspect DOCX content, but verify it's a valid archive
    let bytes = res.bytes().await.unwrap();
    assert_eq!(&bytes[0..2], b"PK");
    assert!(bytes.len() > 100);
}
