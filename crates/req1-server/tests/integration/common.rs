//! Shared test helpers for integration tests.
#![allow(dead_code)]

use axum::http::StatusCode;
use reqwest::Client;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use serde_json::{Value, json};
use std::fmt::Write;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use req1_server::config::Config;
use req1_server::routes;
use req1_server::state::AppState;

use req1_core::service::project_template::ProjectTemplateService;

/// Spin up a test server on a random port and return its base URL.
pub async fn spawn_server() -> String {
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be set");

    let db = Database::connect(&database_url)
        .await
        .expect("failed to connect to test database");

    migration::Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");

    let _ = ProjectTemplateService::seed_builtins(&db).await;

    let config = Config {
        database_url,
        redis_url: None,
        listen_addr: "127.0.0.1:0".to_string(),
        cors_origin: None,
        static_dir: None,
        build_sha: None,
        jwt_secret: "test-secret".to_string(),
        jwt_expiration_hours: 24,
    };

    let state = AppState { db, config };
    let app = routes::router(state).layer(CorsLayer::permissive());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind");
    let addr: SocketAddr = listener.local_addr().expect("failed to get local addr");

    let _ = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    format!("http://{addr}")
}

pub fn api(base: &str) -> String {
    format!("{base}/api/v1")
}

/// Register a test user, log in, and return a reqwest Client with the Bearer
/// token set as a default header so every request is authenticated.
pub async fn authed_client(base: &str) -> Client {
    let anon = Client::new();
    let url = format!("{}/auth", api(base));

    let _ = anon
        .post(format!("{url}/register"))
        .json(&json!({
            "email": "test@example.com",
            "password": "password123",
            "display_name": "Test User"
        }))
        .send()
        .await
        .unwrap();

    let res = anon
        .post(format!("{url}/login"))
        .json(&json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    let body: Value = res.json().await.unwrap();
    let token = body["token"].as_str().expect("login must return token");

    let mut headers = reqwest::header::HeaderMap::new();
    let _ = headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );

    Client::builder().default_headers(headers).build().unwrap()
}

pub async fn create_workspace(client: &Client, base: &str) -> Value {
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

pub async fn create_project(client: &Client, base: &str) -> (Value, Value) {
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

pub async fn create_module(client: &Client, base: &str) -> (Value, Value, Value) {
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

pub async fn create_two_objects(client: &Client, base: &str) -> (String, String, String) {
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

pub async fn create_link_type(client: &Client, base: &str) -> String {
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

pub async fn create_object(client: &Client, base: &str, mod_id: &str, heading: &str) -> Value {
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

pub async fn create_dashboard(client: &Client, base: &str, workspace_id: &str) -> Value {
    let res = client
        .post(format!(
            "{}/workspaces/{workspace_id}/dashboards",
            api(base)
        ))
        .json(&json!({
            "name": format!("dash-{}", uuid::Uuid::now_v7()),
            "description": "test dashboard"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    res.json().await.unwrap()
}

/// Build a minimal valid DOCX file (ZIP) containing `word/document.xml`.
pub fn build_test_docx(paragraphs: &[(&str, &str, Option<&str>)]) -> Vec<u8> {
    use std::io::{Cursor, Write as _};

    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#).unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#).unwrap();

    zip.start_file("word/_rels/document.xml.rels", options)
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#,
    )
    .unwrap();

    let mut doc = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
<w:body>"#,
    );

    for (style, text, bookmark) in paragraphs {
        doc.push_str("<w:p>");
        let _ = write!(doc, r#"<w:pPr><w:pStyle w:val="{style}"/></w:pPr>"#);
        if let Some(bm) = bookmark {
            let _ = write!(
                doc,
                r#"<w:bookmarkStart w:id="0" w:name="{bm}"/><w:bookmarkEnd w:id="0"/>"#,
            );
        }
        let _ = write!(doc, r"<w:r><w:t>{text}</w:t></w:r>");
        doc.push_str("</w:p>");
    }

    doc.push_str("</w:body></w:document>");

    zip.start_file("word/document.xml", options).unwrap();
    zip.write_all(doc.as_bytes()).unwrap();

    let result = zip.finish().unwrap();
    result.into_inner()
}
