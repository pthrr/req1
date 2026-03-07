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

mod integration {
    pub mod common;

    mod attributes;
    mod baselines;
    mod dashboards;
    mod docx_import;
    mod health;
    mod impact;
    mod links;
    mod modules;
    mod objects;
    mod publish;
    mod reviews;
    mod scripts;
    mod templates;
    mod users;
    mod workspaces;
}
