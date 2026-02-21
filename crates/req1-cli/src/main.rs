use std::fmt::Write as _;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "req1", about = "req1 requirements management CLI")]
struct Cli {
    /// Server base URL
    #[arg(long, env = "REQ1_URL", default_value = "http://localhost:3000")]
    url: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List resources
    List {
        #[command(subcommand)]
        resource: ListResource,
    },
    /// Create a resource
    Create {
        #[command(subcommand)]
        resource: CreateResource,
    },
    /// Update a resource
    Update {
        #[command(subcommand)]
        resource: UpdateResource,
    },
    /// Delete a resource
    Delete {
        #[command(subcommand)]
        resource: DeleteResource,
    },
    /// Publish a module to HTML
    Publish {
        /// Module ID
        #[arg(long)]
        module_id: String,
        /// Output format
        #[arg(long, default_value = "html")]
        format: String,
        /// Output file path
        #[arg(long, short)]
        output: String,
    },
    /// Validate module objects (server-side rules)
    Validate {
        /// Module ID
        #[arg(long)]
        module_id: String,
    },
    /// Mark objects as reviewed
    Review {
        /// Module ID
        #[arg(long)]
        module_id: String,
        /// Object ID (if omitted, reviews all unreviewed objects)
        #[arg(long)]
        object_id: Option<String>,
    },
    /// Resolve a suspect link
    ResolveSuspect {
        /// Link ID
        #[arg(long)]
        link_id: String,
    },
}

#[derive(Subcommand)]
enum ListResource {
    /// List modules
    Modules {
        #[arg(long)]
        project_id: Option<String>,
    },
    /// List objects in a module
    Objects {
        #[arg(long)]
        module_id: String,
        /// Show as indented tree
        #[arg(long)]
        tree: bool,
        /// Output format: table (default) or json
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// List links
    Links {
        #[arg(long)]
        module_id: Option<String>,
    },
    /// List link types
    LinkTypes,
}

#[derive(Subcommand)]
enum CreateResource {
    /// Create an object in a module
    Object {
        #[arg(long)]
        module_id: String,
        #[arg(long)]
        heading: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long, default_value = "normative")]
        classification: String,
        #[arg(long)]
        parent_id: Option<String>,
    },
    /// Create a link between objects
    Link {
        #[arg(long)]
        source: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        link_type_id: String,
    },
    /// Create a link type
    LinkType {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
enum UpdateResource {
    /// Update an object
    Object {
        #[arg(long)]
        module_id: String,
        #[arg(long)]
        object_id: String,
        #[arg(long)]
        heading: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        classification: Option<String>,
    },
}

#[derive(Subcommand)]
enum DeleteResource {
    /// Delete an object
    Object {
        #[arg(long)]
        module_id: String,
        #[arg(long)]
        object_id: String,
    },
    /// Delete a link
    Link {
        #[arg(long)]
        link_id: String,
    },
}

#[derive(Debug, Deserialize)]
struct PaginatedResponse<T> {
    items: Vec<T>,
    #[serde(rename = "total")]
    _total: u64,
}

#[derive(Debug, Deserialize)]
struct Module {
    id: String,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReqObject {
    id: String,
    level: String,
    heading: Option<String>,
    body: Option<String>,
    classification: String,
    current_version: i32,
    parent_id: Option<String>,
    position: i32,
    attributes: Option<serde_json::Value>,
    content_fingerprint: String,
    reviewed_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)]
struct Link {
    id: String,
    source_object_id: String,
    target_object_id: String,
    link_type_id: String,
    suspect: bool,
}

#[derive(Debug, Deserialize)]
struct LinkType {
    id: String,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ValidationIssue {
    rule: String,
    severity: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ValidationReport {
    issues: Vec<ValidationIssue>,
    object_count: usize,
    link_count: usize,
}

fn obj_needs_review(o: &ReqObject) -> bool {
    o.reviewed_fingerprint
        .as_ref()
        .is_none_or(|fp| *fp != o.content_fingerprint)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let base = cli.url.trim_end_matches('/');

    match cli.command {
        Command::List { resource } => cmd_list(&client, base, resource).await?,
        Command::Create { resource } => cmd_create(&client, base, resource).await?,
        Command::Update { resource } => cmd_update(&client, base, resource).await?,
        Command::Delete { resource } => cmd_delete(&client, base, resource).await?,
        Command::Publish {
            module_id,
            format,
            output,
        } => cmd_publish(&client, base, &module_id, &format, &output).await?,
        Command::Validate { module_id } => cmd_validate(&client, base, &module_id).await?,
        Command::Review {
            module_id,
            object_id,
        } => cmd_review(&client, base, &module_id, object_id.as_deref()).await?,
        Command::ResolveSuspect { link_id } => {
            cmd_resolve_suspect(&client, base, &link_id).await?;
        }
    }

    Ok(())
}

async fn cmd_list(client: &reqwest::Client, base: &str, resource: ListResource) -> Result<()> {
    match resource {
        ListResource::Modules { project_id } => {
            cmd_list_modules(client, base, project_id.as_deref()).await
        }
        ListResource::Objects {
            module_id,
            tree,
            format,
        } => cmd_list_objects(client, base, &module_id, tree, &format).await,
        ListResource::Links { module_id } => {
            cmd_list_links(client, base, module_id.as_deref()).await
        }
        ListResource::LinkTypes => cmd_list_link_types(client, base).await,
    }
}

async fn cmd_list_modules(
    client: &reqwest::Client,
    base: &str,
    project_id: Option<&str>,
) -> Result<()> {
    let mut url = format!("{base}/api/v1/modules?limit=500");
    if let Some(pid) = project_id {
        let _ = write!(url, "&project_id={pid}");
    }
    let resp: PaginatedResponse<Module> = client
        .get(&url)
        .send()
        .await
        .context("request failed")?
        .json()
        .await
        .context("invalid json")?;

    println!("{:<38} {:<30} {:<20}", "ID", "NAME", "DESCRIPTION");
    println!("{}", "-".repeat(90));
    for m in &resp.items {
        println!(
            "{:<38} {:<30} {:<20}",
            m.id,
            m.name,
            m.description.as_deref().unwrap_or("")
        );
    }
    println!("\n{} module(s)", resp.items.len());
    Ok(())
}

async fn cmd_list_objects(
    client: &reqwest::Client,
    base: &str,
    module_id: &str,
    tree: bool,
    format: &str,
) -> Result<()> {
    let url = format!("{base}/api/v1/modules/{module_id}/objects?limit=500");
    let resp: PaginatedResponse<ReqObject> = client
        .get(&url)
        .send()
        .await
        .context("request failed")?
        .json()
        .await
        .context("invalid json")?;

    if format == "json" {
        let json = serde_json::to_string_pretty(&resp.items).context("serialize json")?;
        println!("{json}");
        return Ok(());
    }

    if tree {
        print_tree(&resp.items);
        println!("\n{} object(s)", resp.items.len());
        return Ok(());
    }

    println!(
        "{:<10} {:<38} {:<30} {:<12} {:<5} {:<10}",
        "LEVEL", "ID", "HEADING", "CLASS", "VER", "REVIEWED"
    );
    println!("{}", "-".repeat(110));
    for o in &resp.items {
        let reviewed = !obj_needs_review(o);
        println!(
            "{:<10} {:<38} {:<30} {:<12} {:<5} {:<10}",
            o.level,
            o.id,
            o.heading.as_deref().unwrap_or(""),
            o.classification,
            o.current_version,
            if reviewed { "yes" } else { "no" }
        );
    }
    println!("\n{} object(s)", resp.items.len());
    Ok(())
}

fn print_tree_node(
    obj: &ReqObject,
    depth: usize,
    children_map: &std::collections::HashMap<Option<&str>, Vec<&ReqObject>>,
) {
    let indent = "  ".repeat(depth);
    let reviewed = if obj
        .reviewed_fingerprint
        .as_ref()
        .is_some_and(|fp| *fp == obj.content_fingerprint)
    {
        "[R]"
    } else {
        "[ ]"
    };
    println!(
        "{indent}{reviewed} {level} {heading}",
        level = obj.level,
        heading = obj.heading.as_deref().unwrap_or(""),
    );
    if let Some(kids) = children_map.get(&Some(obj.id.as_str())) {
        for kid in kids {
            print_tree_node(kid, depth + 1, children_map);
        }
    }
}

fn print_tree(objects: &[ReqObject]) {
    let mut children_map: std::collections::HashMap<Option<&str>, Vec<&ReqObject>> =
        std::collections::HashMap::new();
    for obj in objects {
        children_map
            .entry(obj.parent_id.as_deref())
            .or_default()
            .push(obj);
    }
    for group in children_map.values_mut() {
        group.sort_by_key(|o| o.position);
    }

    if let Some(roots) = children_map.get(&None) {
        for root in roots {
            print_tree_node(root, 0, &children_map);
        }
    }
}

async fn cmd_list_links(
    client: &reqwest::Client,
    base: &str,
    module_id: Option<&str>,
) -> Result<()> {
    let mut url = format!("{base}/api/v1/links?limit=500");
    if let Some(mid) = module_id {
        let _ = write!(url, "&module_id={mid}");
    }
    let resp: PaginatedResponse<Link> = client
        .get(&url)
        .send()
        .await
        .context("request failed")?
        .json()
        .await
        .context("invalid json")?;

    println!(
        "{:<38} {:<38} {:<38} {:<38} {:<8}",
        "ID", "SOURCE", "TARGET", "TYPE", "SUSPECT"
    );
    println!("{}", "-".repeat(145));
    for l in &resp.items {
        println!(
            "{:<38} {:<38} {:<38} {:<38} {:<8}",
            l.id,
            l.source_object_id,
            l.target_object_id,
            l.link_type_id,
            if l.suspect { "yes" } else { "no" }
        );
    }
    println!("\n{} link(s)", resp.items.len());
    Ok(())
}

async fn cmd_list_link_types(client: &reqwest::Client, base: &str) -> Result<()> {
    let url = format!("{base}/api/v1/link-types");
    let items: Vec<LinkType> = client
        .get(&url)
        .send()
        .await
        .context("request failed")?
        .json()
        .await
        .context("invalid json")?;

    println!("{:<38} {:<30} {:<30}", "ID", "NAME", "DESCRIPTION");
    println!("{}", "-".repeat(100));
    for lt in &items {
        println!(
            "{:<38} {:<30} {:<30}",
            lt.id,
            lt.name,
            lt.description.as_deref().unwrap_or("")
        );
    }
    println!("\n{} link type(s)", items.len());
    Ok(())
}

async fn cmd_create(client: &reqwest::Client, base: &str, resource: CreateResource) -> Result<()> {
    match resource {
        CreateResource::Object {
            module_id,
            heading,
            body,
            classification,
            parent_id,
        } => {
            cmd_create_object(
                client,
                base,
                CreateObjectArgs {
                    module_id: &module_id,
                    heading: heading.as_deref(),
                    body: body.as_deref(),
                    classification: &classification,
                    parent_id: parent_id.as_deref(),
                },
            )
            .await
        }
        CreateResource::Link {
            source,
            target,
            link_type_id,
        } => cmd_create_link(client, base, &source, &target, &link_type_id).await,
        CreateResource::LinkType { name, description } => {
            cmd_create_link_type(client, base, &name, description.as_deref()).await
        }
    }
}

struct CreateObjectArgs<'a> {
    module_id: &'a str,
    heading: Option<&'a str>,
    body: Option<&'a str>,
    classification: &'a str,
    parent_id: Option<&'a str>,
}

async fn cmd_create_object(
    client: &reqwest::Client,
    base: &str,
    args: CreateObjectArgs<'_>,
) -> Result<()> {
    let url = format!("{base}/api/v1/modules/{}/objects", args.module_id);
    let mut payload = serde_json::Map::new();
    if let Some(h) = args.heading {
        let _ = payload.insert("heading".to_owned(), serde_json::json!(h));
    }
    if let Some(b) = args.body {
        let _ = payload.insert("body".to_owned(), serde_json::json!(b));
    }
    let _ = payload.insert(
        "classification".to_owned(),
        serde_json::json!(args.classification),
    );
    if let Some(pid) = args.parent_id {
        let _ = payload.insert("parent_id".to_owned(), serde_json::json!(pid));
    }

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("request failed")?;

    ensure_success(&resp)?;
    let obj: ReqObject = resp.json().await.context("invalid json")?;
    println!("Created object {} (level {})", obj.id, obj.level);
    Ok(())
}

async fn cmd_create_link(
    client: &reqwest::Client,
    base: &str,
    source: &str,
    target: &str,
    link_type_id: &str,
) -> Result<()> {
    let url = format!("{base}/api/v1/links");
    let payload = serde_json::json!({
        "source_object_id": source,
        "target_object_id": target,
        "link_type_id": link_type_id,
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("request failed")?;

    ensure_success(&resp)?;
    let link: Link = resp.json().await.context("invalid json")?;
    println!("Created link {}", link.id);
    Ok(())
}

async fn cmd_create_link_type(
    client: &reqwest::Client,
    base: &str,
    name: &str,
    description: Option<&str>,
) -> Result<()> {
    let url = format!("{base}/api/v1/link-types");
    let mut payload = serde_json::Map::new();
    let _ = payload.insert("name".to_owned(), serde_json::json!(name));
    if let Some(d) = description {
        let _ = payload.insert("description".to_owned(), serde_json::json!(d));
    }

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("request failed")?;

    ensure_success(&resp)?;
    let lt: LinkType = resp.json().await.context("invalid json")?;
    println!("Created link type {} ({})", lt.id, lt.name);
    Ok(())
}

async fn cmd_update(client: &reqwest::Client, base: &str, resource: UpdateResource) -> Result<()> {
    match resource {
        UpdateResource::Object {
            module_id,
            object_id,
            heading,
            body,
            classification,
        } => {
            let url = format!("{base}/api/v1/modules/{module_id}/objects/{object_id}");
            let mut payload = serde_json::Map::new();
            if let Some(h) = heading {
                let _ = payload.insert("heading".to_owned(), serde_json::json!(h));
            }
            if let Some(b) = body {
                let _ = payload.insert("body".to_owned(), serde_json::json!(b));
            }
            if let Some(c) = classification {
                let _ = payload.insert("classification".to_owned(), serde_json::json!(c));
            }

            let resp = client
                .patch(&url)
                .json(&payload)
                .send()
                .await
                .context("request failed")?;

            ensure_success(&resp)?;
            let obj: ReqObject = resp.json().await.context("invalid json")?;
            println!("Updated object {} (v{})", obj.id, obj.current_version);
            Ok(())
        }
    }
}

async fn cmd_delete(client: &reqwest::Client, base: &str, resource: DeleteResource) -> Result<()> {
    match resource {
        DeleteResource::Object {
            module_id,
            object_id,
        } => {
            let url = format!("{base}/api/v1/modules/{module_id}/objects/{object_id}");
            let resp = client.delete(&url).send().await.context("request failed")?;
            ensure_success(&resp)?;
            println!("Deleted object {object_id}");
            Ok(())
        }
        DeleteResource::Link { link_id } => {
            let url = format!("{base}/api/v1/links/{link_id}");
            let resp = client.delete(&url).send().await.context("request failed")?;
            ensure_success(&resp)?;
            println!("Deleted link {link_id}");
            Ok(())
        }
    }
}

async fn cmd_publish(
    client: &reqwest::Client,
    base: &str,
    module_id: &str,
    format: &str,
    output: &str,
) -> Result<()> {
    let url = format!("{base}/api/v1/modules/{module_id}/publish?format={format}");
    let resp = client.get(&url).send().await.context("request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("publish failed ({status}): {body}");
    }

    let content = resp.text().await.context("read response")?;
    std::fs::write(output, &content).with_context(|| format!("write to {output}"))?;
    println!("Published to {output} ({} bytes)", content.len());
    Ok(())
}

async fn cmd_validate(client: &reqwest::Client, base: &str, module_id: &str) -> Result<()> {
    let url = format!("{base}/api/v1/modules/{module_id}/validate");
    let resp = client.get(&url).send().await.context("request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("validation request failed ({status}): {body}");
    }

    let report: ValidationReport = resp.json().await.context("invalid json")?;

    println!(
        "Checked {} objects, {} links\n",
        report.object_count, report.link_count
    );

    if report.issues.is_empty() {
        println!("All checks passed.");
    } else {
        let errors = report
            .issues
            .iter()
            .filter(|i| i.severity == "error")
            .count();
        let warnings = report
            .issues
            .iter()
            .filter(|i| i.severity == "warning")
            .count();
        let infos = report
            .issues
            .iter()
            .filter(|i| i.severity == "info")
            .count();
        println!(
            "{} issues: {} errors, {} warnings, {} info\n",
            report.issues.len(),
            errors,
            warnings,
            infos
        );
        for issue in &report.issues {
            println!(
                "  [{:<7}] {:<20} {}",
                issue.severity, issue.rule, issue.message
            );
        }
        if errors > 0 {
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn cmd_review(
    client: &reqwest::Client,
    base: &str,
    module_id: &str,
    object_id: Option<&str>,
) -> Result<()> {
    if let Some(oid) = object_id {
        review_single(client, base, module_id, oid).await?;
    } else {
        // Review all unreviewed objects
        let url = format!("{base}/api/v1/modules/{module_id}/objects?limit=500&needs_review=true");
        let resp: PaginatedResponse<ReqObject> = client
            .get(&url)
            .send()
            .await
            .context("request failed")?
            .json()
            .await
            .context("invalid json")?;

        if resp.items.is_empty() {
            println!("All objects are already reviewed.");
            return Ok(());
        }

        let count = resp.items.len();
        for o in &resp.items {
            review_single(client, base, module_id, &o.id).await?;
        }
        println!("Reviewed {count} object(s).");
    }
    Ok(())
}

async fn review_single(
    client: &reqwest::Client,
    base: &str,
    module_id: &str,
    object_id: &str,
) -> Result<()> {
    let url = format!("{base}/api/v1/modules/{module_id}/objects/{object_id}");
    let payload = serde_json::json!({"reviewed": true});
    let resp = client
        .patch(&url)
        .json(&payload)
        .send()
        .await
        .context("request failed")?;

    ensure_success(&resp)?;
    let obj: ReqObject = resp.json().await.context("invalid json")?;
    println!(
        "Reviewed [{}] {}",
        obj.level,
        obj.heading.as_deref().unwrap_or(&obj.id)
    );
    Ok(())
}

async fn cmd_resolve_suspect(client: &reqwest::Client, base: &str, link_id: &str) -> Result<()> {
    let url = format!("{base}/api/v1/links/{link_id}");
    let payload = serde_json::json!({"suspect": false});
    let resp = client
        .patch(&url)
        .json(&payload)
        .send()
        .await
        .context("request failed")?;

    ensure_success(&resp)?;
    let link: Link = resp.json().await.context("invalid json")?;
    println!(
        "Resolved suspect link {} ({} -> {})",
        link.id, link.source_object_id, link.target_object_id
    );
    Ok(())
}

fn ensure_success(resp: &reqwest::Response) -> Result<()> {
    if !resp.status().is_success() {
        anyhow::bail!("request failed with status {}", resp.status());
    }
    Ok(())
}
