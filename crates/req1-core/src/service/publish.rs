use std::fmt::Write as _;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder};
use serde::Serialize;

use entity::{attribute_definition, object};
use std::io::Write as _;

use regex::Regex;

use crate::error::CoreError;

const DEFAULT_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{{ module_name }}</title>
  <style>
    body { font-family: system-ui, -apple-system, sans-serif; max-width: 900px; margin: 40px auto; padding: 0 20px; color: #212121; }
    h1 { border-bottom: 2px solid #1976d2; padding-bottom: 8px; }
    .object { margin-bottom: 1.5em; }
    .object-heading { color: #1565c0; }
    .object-body { margin-top: 0.5em; }
    .object-meta { font-size: 0.8em; color: #757575; margin-top: 0.25em; }
    .informative { border-left: 3px solid #1565c0; padding-left: 12px; }
    .heading-only { border-left: 3px solid #6a1b9a; padding-left: 12px; }
    table { border-collapse: collapse; width: 100%; margin: 0.5em 0; }
    th, td { border: 1px solid #ccc; padding: 6px 10px; text-align: left; }
    th { background: #f5f5f5; }
    code { background: #f5f5f5; padding: 2px 4px; border-radius: 3px; }
    pre { background: #f5f5f5; padding: 12px; border-radius: 4px; overflow-x: auto; }
    @media print { body { max-width: none; margin: 0; } }
  </style>
</head>
<body>
  <h1>{{ module_name }}</h1>
  <p>Generated: {{ generated_at }}</p>
  <hr>
  {% for obj in objects %}
  <div id="obj-{{ obj.id }}" class="object{% if obj.classification == 'informative' %} informative{% elif obj.classification == 'heading' %} heading-only{% endif %}">
    {% if obj.heading %}
    <{{ obj.heading_tag }} class="object-heading">{% if prefix %}{{ prefix }}{{ separator }}{% endif %}{{ obj.level }} {{ obj.heading }}</{{ obj.heading_tag }}>
    {% endif %}
    {% if obj.body_html %}
    <div class="object-body">{{ obj.body_html }}</div>
    {% endif %}
    <div class="object-meta">v{{ obj.version }} | {{ obj.classification }} | <a href="#obj-{{ obj.id }}">link</a></div>
  </div>
  {% endfor %}
  <script>
    if (location.hash) {
      var el = document.getElementById(location.hash.slice(1));
      if (el) el.scrollIntoView({ behavior: 'smooth' });
    }
  </script>
</body>
</html>"##;

struct PublishData {
    module: entity::module::Model,
    objects: Vec<object::Model>,
}

async fn load_publish_data(
    db: &impl ConnectionTrait,
    module_id: uuid::Uuid,
) -> Result<PublishData, CoreError> {
    let module = entity::module::Entity::find_by_id(module_id)
        .one(db)
        .await?
        .ok_or_else(|| CoreError::NotFound(format!("module {module_id} not found")))?;

    let objects = object::Entity::find()
        .filter(object::Column::ModuleId.eq(module_id))
        .filter(object::Column::DeletedAt.is_null())
        .order_by(object::Column::Level, Order::Asc)
        .all(db)
        .await?;

    Ok(PublishData { module, objects })
}

pub struct PublishService;

impl PublishService {
    pub async fn render_html(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<String, CoreError> {
        let data = load_publish_data(db, module_id).await?;

        let template_src = data
            .module
            .publish_template
            .as_deref()
            .unwrap_or(DEFAULT_TEMPLATE);

        let mut env = minijinja::Environment::new();
        env.add_template("doc", template_src)
            .map_err(|e| CoreError::Internal(format!("template error: {e}")))?;

        let tmpl = env
            .get_template("doc")
            .map_err(|e| CoreError::Internal(format!("template error: {e}")))?;

        let obj_data: Vec<minijinja::Value> = data
            .objects
            .iter()
            .map(|o| {
                let depth = o.level.matches('.').count();
                let heading_tag = format!("h{}", (depth + 2).min(6));

                let body_html = o
                    .body
                    .as_deref()
                    .map(|b| {
                        if is_html_content(b) {
                            process_plantuml_blocks(b)
                        } else {
                            let processed = process_plantuml_blocks(b);
                            markdown_to_html(&processed)
                        }
                    })
                    .unwrap_or_default();

                minijinja::context! {
                    id => o.id,
                    level => o.level,
                    heading => o.heading,
                    heading_tag => heading_tag,
                    body_html => body_html,
                    version => o.current_version,
                    classification => o.classification,
                }
            })
            .collect();

        let ctx = minijinja::context! {
            module_name => data.module.name,
            prefix => data.module.prefix,
            separator => data.module.separator,
            digits => data.module.digits,
            generated_at => chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
            objects => obj_data,
        };

        tmpl.render(ctx)
            .map_err(|e| CoreError::Internal(format!("render error: {e}")))
    }

    pub async fn render_markdown(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<String, CoreError> {
        let data = load_publish_data(db, module_id).await?;
        let mut out = String::new();

        let _ = writeln!(out, "# {}", data.module.name);
        let _ = writeln!(
            out,
            "\nGenerated: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        );

        for o in &data.objects {
            let depth = o.level.matches('.').count() + 2;
            let hashes = "#".repeat(depth.min(6));

            if let Some(heading) = &o.heading {
                let _ = writeln!(out, "{hashes} {} {heading}", o.level);
            }

            if o.classification != "normative" {
                let _ = writeln!(out, "\n*Classification: {}*", o.classification);
            }

            if let Some(body) = &o.body {
                if is_html_content(body) {
                    let stripped = strip_html_tags(body);
                    let _ = writeln!(out, "\n{stripped}");
                } else {
                    let _ = writeln!(out, "\n{body}");
                }
            }

            out.push('\n');
        }

        Ok(out)
    }

    pub async fn render_latex(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<String, CoreError> {
        let data = load_publish_data(db, module_id).await?;
        let mut out = String::new();

        let _ = writeln!(out, "\\documentclass{{article}}");
        let _ = writeln!(out, "\\usepackage{{hyperref,longtable,geometry}}");
        let _ = writeln!(out, "\\usepackage[utf8]{{inputenc}}");
        let _ = writeln!(out, "\\title{{{}}}", escape_latex(&data.module.name));
        let _ = writeln!(
            out,
            "\\date{{{}}}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        );
        let _ = writeln!(out, "\\begin{{document}}");
        let _ = writeln!(out, "\\maketitle");
        let _ = writeln!(out, "\\tableofcontents");
        let _ = writeln!(out, "\\newpage\n");

        let section_cmds = [
            "\\section",
            "\\subsection",
            "\\subsubsection",
            "\\paragraph",
            "\\subparagraph",
        ];

        for o in &data.objects {
            let depth = o.level.matches('.').count();
            let cmd = section_cmds
                .get(depth.min(section_cmds.len() - 1))
                .unwrap_or(section_cmds.last().expect("non-empty"));

            if let Some(heading) = &o.heading {
                let _ = writeln!(out, "{cmd}{{{}}}", escape_latex(heading));
            }

            if o.classification != "normative" {
                let _ = writeln!(
                    out,
                    "\\textit{{Classification: {}}}\\\\",
                    escape_latex(&o.classification)
                );
            }

            if let Some(body) = &o.body {
                let text = if is_html_content(body) {
                    strip_html_tags(body)
                } else {
                    body.clone()
                };
                let _ = writeln!(out, "\n{}\n", escape_latex(&text));
            }
        }

        let _ = writeln!(out, "\\end{{document}}");
        Ok(out)
    }

    pub async fn render_text(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<String, CoreError> {
        let data = load_publish_data(db, module_id).await?;
        let mut out = String::new();

        let _ = writeln!(out, "{}", data.module.name);
        let _ = writeln!(out, "{}", "=".repeat(data.module.name.len()));
        let _ = writeln!(
            out,
            "Generated: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        );

        for o in &data.objects {
            if let Some(heading) = &o.heading {
                let _ = writeln!(out, "[{}] {heading}", o.level);
            } else {
                let _ = writeln!(out, "[{}]", o.level);
            }

            if o.classification != "normative" {
                let _ = writeln!(out, "  Classification: {}", o.classification);
            }

            if let Some(body) = &o.body {
                if is_html_content(body) {
                    let stripped = strip_html_tags(body);
                    let _ = writeln!(out, "{stripped}");
                } else {
                    let _ = writeln!(out, "{body}");
                }
            }

            out.push('\n');
        }

        Ok(out)
    }

    pub async fn render_csv(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<String, CoreError> {
        let data = load_publish_data(db, module_id).await?;

        let attr_defs = attribute_definition::Entity::find()
            .filter(attribute_definition::Column::ModuleId.eq(module_id))
            .order_by(attribute_definition::Column::Name, Order::Asc)
            .all(db)
            .await?;

        let mut wtr = csv::Writer::from_writer(Vec::new());

        // Header row
        let mut headers: Vec<String> = vec![
            "id".to_owned(),
            "level".to_owned(),
            "heading".to_owned(),
            "body".to_owned(),
            "classification".to_owned(),
            "version".to_owned(),
        ];
        for def in &attr_defs {
            headers.push(def.name.clone());
        }
        wtr.write_record(&headers)
            .map_err(|e| CoreError::Internal(format!("csv header error: {e}")))?;

        // Data rows
        for o in &data.objects {
            let mut row: Vec<String> = vec![
                o.id.to_string(),
                o.level.clone(),
                o.heading.clone().unwrap_or_default(),
                o.body.clone().unwrap_or_default(),
                o.classification.clone(),
                o.current_version.to_string(),
            ];

            let attrs = o
                .attributes
                .as_ref()
                .and_then(serde_json::Value::as_object);
            for def in &attr_defs {
                let val = attrs
                    .and_then(|m| m.get(&def.name))
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                row.push(val);
            }

            wtr.write_record(&row)
                .map_err(|e| CoreError::Internal(format!("csv row error: {e}")))?;
        }

        let bytes = wtr
            .into_inner()
            .map_err(|e| CoreError::Internal(format!("csv flush error: {e}")))?;
        String::from_utf8(bytes).map_err(|e| CoreError::Internal(format!("csv utf8 error: {e}")))
    }

    pub async fn render_yaml(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<String, CoreError> {
        let data = load_publish_data(db, module_id).await?;

        let attr_defs = attribute_definition::Entity::find()
            .filter(attribute_definition::Column::ModuleId.eq(module_id))
            .order_by(attribute_definition::Column::Name, Order::Asc)
            .all(db)
            .await?;

        let attr_def_names: Vec<String> = attr_defs.iter().map(|d| d.name.clone()).collect();

        let yaml_objects: Vec<YamlObject> = data
            .objects
            .iter()
            .map(|o| {
                let mut attributes = serde_json::Map::new();
                if let Some(obj_attrs) = o.attributes.as_ref().and_then(|v| v.as_object()) {
                    for name in &attr_def_names {
                        if let Some(val) = obj_attrs.get(name) {
                            let _ = attributes.insert(name.clone(), val.clone());
                        }
                    }
                }

                YamlObject {
                    id: o.id.to_string(),
                    level: o.level.clone(),
                    heading: o.heading.clone(),
                    body: o.body.clone(),
                    classification: o.classification.clone(),
                    version: o.current_version,
                    attributes: serde_json::Value::Object(attributes),
                }
            })
            .collect();

        let yaml_doc = YamlDocument {
            module: YamlModule {
                name: data.module.name,
                prefix: data.module.prefix,
                separator: data.module.separator,
                digits: data.module.digits,
            },
            objects: yaml_objects,
        };

        serde_yaml::to_string(&yaml_doc)
            .map_err(|e| CoreError::Internal(format!("yaml error: {e}")))
    }

    pub async fn render_pdf(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<Vec<u8>, CoreError> {
        let html = Self::render_html(db, module_id).await?;
        try_wkhtmltopdf(&html)
            .or_else(|_| try_weasyprint(&html))
            .map_err(|e| {
                CoreError::Internal(format!(
                    "PDF generation failed: {e}. Install wkhtmltopdf or weasyprint."
                ))
            })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn render_xlsx(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<Vec<u8>, CoreError> {
        let data = load_publish_data(db, module_id).await?;

        let attr_defs = attribute_definition::Entity::find()
            .filter(attribute_definition::Column::ModuleId.eq(module_id))
            .order_by(attribute_definition::Column::Name, Order::Asc)
            .all(db)
            .await?;

        let mut workbook = rust_xlsxwriter::Workbook::new();

        // Bold header format
        let bold = rust_xlsxwriter::Format::new().set_bold();

        // Requirements sheet
        let sheet = workbook.add_worksheet();
        let _ = sheet
            .set_name("Requirements")
            .map_err(|e| CoreError::Internal(format!("xlsx sheet error: {e}")))?;

        // Headers
        let mut col: u16 = 0;
        let base_headers = ["id", "level", "heading", "body", "classification", "version", "lifecycle_state"];
        for h in &base_headers {
            let _ = sheet.write_string_with_format(0, col, *h, &bold);
            col += 1;
        }
        for def in &attr_defs {
            let _ = sheet.write_string_with_format(0, col, &def.name, &bold);
            col += 1;
        }

        // Data rows
        for (row_idx, o) in data.objects.iter().enumerate() {
            let row = u32::try_from(row_idx + 1).unwrap_or(u32::MAX);
            let _ = sheet.write_string(row, 0, o.id.to_string());
            let _ = sheet.write_string(row, 1, &o.level);
            let _ = sheet.write_string(row, 2, o.heading.as_deref().unwrap_or(""));
            let _ = sheet.write_string(row, 3, o.body.as_deref().unwrap_or(""));
            let _ = sheet.write_string(row, 4, &o.classification);
            let _ = sheet.write_number(row, 5, f64::from(o.current_version));
            let _ = sheet.write_string(
                row,
                6,
                o.lifecycle_state.as_deref().unwrap_or(""),
            );

            let attrs = o
                .attributes
                .as_ref()
                .and_then(serde_json::Value::as_object);
            for (i, def) in attr_defs.iter().enumerate() {
                let attr_col = u16::try_from(base_headers.len() + i).unwrap_or(u16::MAX);
                let val = attrs
                    .and_then(|m| m.get(&def.name))
                    .map_or_else(String::new, |v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    });
                let _ = sheet.write_string(row, attr_col, &val);
            }
        }

        // Metadata sheet
        let meta_sheet = workbook.add_worksheet();
        let _ = meta_sheet
            .set_name("Metadata")
            .map_err(|e| CoreError::Internal(format!("xlsx sheet error: {e}")))?;

        let _ = meta_sheet.write_string_with_format(0, 0, "Property", &bold);
        let _ = meta_sheet.write_string_with_format(0, 1, "Value", &bold);

        let meta_rows: Vec<(&str, String)> = vec![
            ("Module Name", data.module.name.clone()),
            ("Prefix", data.module.prefix.clone()),
            ("Separator", data.module.separator.clone()),
            ("Digits", data.module.digits.to_string()),
            (
                "Generated At",
                chrono::Utc::now()
                    .format("%Y-%m-%d %H:%M UTC")
                    .to_string(),
            ),
            ("Object Count", data.objects.len().to_string()),
        ];

        for (i, (key, val)) in meta_rows.iter().enumerate() {
            let row = u32::try_from(i + 1).unwrap_or(u32::MAX);
            let _ = meta_sheet.write_string(row, 0, *key);
            let _ = meta_sheet.write_string(row, 1, val);
        }

        workbook
            .save_to_buffer()
            .map_err(|e| CoreError::Internal(format!("xlsx save error: {e}")))
    }

    pub async fn render_docx(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<Vec<u8>, CoreError> {
        let data = load_publish_data(db, module_id).await?;

        let mut docx = docx_rs::Docx::new();

        // Title
        let title_para = docx_rs::Paragraph::new()
            .add_run(docx_rs::Run::new().add_text(&data.module.name).bold())
            .style("Heading1");
        docx = docx.add_paragraph(title_para);

        // Generated timestamp
        let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
        let ts_para = docx_rs::Paragraph::new()
            .add_run(docx_rs::Run::new().add_text(format!("Generated: {ts}")));
        docx = docx.add_paragraph(ts_para);

        // Objects — embed bookmarks for round-trip tracking
        let mut bookmark_counter: usize = 0;
        for o in &data.objects {
            let depth = o.level.matches('.').count();
            let heading_style = match depth {
                0 => "Heading2",
                1 => "Heading3",
                2 => "Heading4",
                _ => "Heading5",
            };

            if let Some(heading) = &o.heading {
                let heading_text = format!("{} {heading}", o.level);
                let bookmark_name = format!("req1_{}", o.id);
                let bm_id = bookmark_counter;
                bookmark_counter += 1;
                let heading_para = docx_rs::Paragraph::new()
                    .add_bookmark_start(bm_id, &bookmark_name)
                    .add_run(docx_rs::Run::new().add_text(heading_text))
                    .add_bookmark_end(bm_id)
                    .style(heading_style);
                docx = docx.add_paragraph(heading_para);
            }

            if let Some(body) = &o.body {
                let text = if is_html_content(body) {
                    strip_html_tags(body)
                } else {
                    body.clone()
                };
                // Split into paragraphs on blank lines
                for line in text.split('\n') {
                    let para = docx_rs::Paragraph::new()
                        .add_run(docx_rs::Run::new().add_text(line));
                    docx = docx.add_paragraph(para);
                }
            }
        }

        let mut buf = std::io::Cursor::new(Vec::new());
        docx.build().pack(&mut buf)
            .map_err(|e| CoreError::Internal(format!("docx build error: {e}")))?;
        Ok(buf.into_inner())
    }
}

fn is_html_content(body: &str) -> bool {
    body.contains("<p>")
        || body.contains("<h1")
        || body.contains("<table")
        || body.contains("<ul>")
        || body.contains("<ol>")
        || body.contains("<img ")
        || body.contains("<blockquote>")
        || body.contains("<pre>")
}

fn strip_html_tags(html: &str) -> String {
    let re = Regex::new(r"<[^>]+>").expect("valid regex");
    re.replace_all(html, "").into_owned()
}

fn process_plantuml_blocks(body: &str) -> String {
    let re = Regex::new(r"(?s)@startuml.*?@enduml").expect("valid regex");
    let mut result = body.to_owned();
    for m in re.find_iter(body) {
        let source = m.as_str();
        let replacement = if let Ok(svg) = try_plantuml_cli(source) {
            format!(r#"<div class="plantuml-diagram">{svg}</div>"#)
        } else {
            let escaped = source.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
            format!(r#"<pre class="plantuml-error">{escaped}</pre>"#)
        };
        result = result.replacen(source, &replacement, 1);
    }
    result
}

fn try_plantuml_cli(source: &str) -> Result<String, String> {
    let mut child = std::process::Command::new("plantuml")
        .args(["-tsvg", "-pipe"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("plantuml: {e}"))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(source.as_bytes())
            .map_err(|e| format!("plantuml stdin: {e}"))?;
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .map_err(|e| format!("plantuml wait: {e}"))?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| format!("plantuml utf8: {e}"))
    } else {
        Err(format!(
            "plantuml failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{Parser, html};
    let parser = Parser::new(md);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn escape_latex(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\textbackslash{}"),
            '&' => out.push_str("\\&"),
            '%' => out.push_str("\\%"),
            '$' => out.push_str("\\$"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            other => out.push(other),
        }
    }
    out
}

#[derive(Serialize)]
struct YamlDocument {
    module: YamlModule,
    objects: Vec<YamlObject>,
}

#[derive(Serialize)]
struct YamlModule {
    name: String,
    prefix: String,
    separator: String,
    digits: i32,
}

#[derive(Serialize)]
struct YamlObject {
    id: String,
    level: String,
    heading: Option<String>,
    body: Option<String>,
    classification: String,
    version: i32,
    attributes: serde_json::Value,
}

fn try_wkhtmltopdf(html: &str) -> Result<Vec<u8>, String> {
    let mut child = std::process::Command::new("wkhtmltopdf")
        .args(["--quiet", "-", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("wkhtmltopdf: {e}"))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(html.as_bytes())
            .map_err(|e| format!("wkhtmltopdf stdin: {e}"))?;
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .map_err(|e| format!("wkhtmltopdf wait: {e}"))?;

    if output.status.success() || !output.stdout.is_empty() {
        Ok(output.stdout)
    } else {
        Err(format!(
            "wkhtmltopdf failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn try_weasyprint(html: &str) -> Result<Vec<u8>, String> {
    let mut child = std::process::Command::new("weasyprint")
        .args(["-", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("weasyprint: {e}"))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(html.as_bytes())
            .map_err(|e| format!("weasyprint stdin: {e}"))?;
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .map_err(|e| format!("weasyprint wait: {e}"))?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(format!(
            "weasyprint failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
