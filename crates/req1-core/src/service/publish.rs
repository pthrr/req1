use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder};

use entity::object;

use crate::error::CoreError;

const DEFAULT_TEMPLATE: &str = r#"<!DOCTYPE html>
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
  <div class="object{% if obj.classification == 'informative' %} informative{% elif obj.classification == 'heading' %} heading-only{% endif %}">
    {% if obj.heading %}
    <{{ obj.heading_tag }} class="object-heading">{% if prefix %}{{ prefix }}{{ separator }}{% endif %}{{ obj.level }} {{ obj.heading }}</{{ obj.heading_tag }}>
    {% endif %}
    {% if obj.body_html %}
    <div class="object-body">{{ obj.body_html }}</div>
    {% endif %}
    <div class="object-meta">v{{ obj.version }} | {{ obj.classification }}</div>
  </div>
  {% endfor %}
</body>
</html>"#;

pub struct PublishService;

impl PublishService {
    pub async fn render_html(
        db: &impl ConnectionTrait,
        module_id: uuid::Uuid,
    ) -> Result<String, CoreError> {
        let module = entity::module::Entity::find_by_id(module_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("module {module_id} not found")))?;

        let objects = object::Entity::find()
            .filter(object::Column::ModuleId.eq(module_id))
            .order_by(object::Column::Level, Order::Asc)
            .all(db)
            .await?;

        let mut env = minijinja::Environment::new();
        env.add_template("doc", DEFAULT_TEMPLATE)
            .map_err(|e| CoreError::Internal(format!("template error: {e}")))?;

        let tmpl = env
            .get_template("doc")
            .map_err(|e| CoreError::Internal(format!("template error: {e}")))?;

        let obj_data: Vec<minijinja::Value> = objects
            .iter()
            .map(|o| {
                let depth = o.level.matches('.').count();
                let heading_tag = format!("h{}", (depth + 2).min(6));

                let body_html = o.body.as_deref().map(markdown_to_html).unwrap_or_default();

                minijinja::context! {
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
            module_name => module.name,
            prefix => module.prefix,
            separator => module.separator,
            digits => module.digits,
            generated_at => chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
            objects => obj_data,
        };

        tmpl.render(ctx)
            .map_err(|e| CoreError::Internal(format!("render error: {e}")))
    }
}

fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{Parser, html};
    let parser = Parser::new(md);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
