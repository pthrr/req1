use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder};
use serde::Serialize;
use uuid::Uuid;

use entity::attribute_definition;

use crate::error::CoreError;
use crate::service::object::{CreateObjectInput, ObjectService};

#[derive(Debug, Serialize)]
pub struct CsvImportResult {
    pub objects_created: usize,
}

pub struct CsvImportService;

impl CsvImportService {
    pub async fn import_csv(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        csv_content: &str,
    ) -> Result<CsvImportResult, CoreError> {
        // Verify module exists
        let _module = entity::module::Entity::find_by_id(module_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("module {module_id} not found")))?;

        let attr_defs = attribute_definition::Entity::find()
            .filter(attribute_definition::Column::ModuleId.eq(module_id))
            .order_by(attribute_definition::Column::Name, Order::Asc)
            .all(db)
            .await?;

        let attr_def_names: Vec<String> = attr_defs.iter().map(|d| d.name.clone()).collect();

        let mut rdr = csv::Reader::from_reader(csv_content.as_bytes());

        let headers = rdr
            .headers()
            .map_err(|e| CoreError::BadRequest(format!("csv header error: {e}")))?
            .clone();

        // Find column indices for required fields
        let level_idx = find_column(&headers, "level")?;
        let heading_idx = headers.iter().position(|h| h == "heading");
        let body_idx = headers.iter().position(|h| h == "body");
        let classification_idx = headers.iter().position(|h| h == "classification");

        // Find attribute column indices
        let attr_columns: Vec<(usize, String)> = headers
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                if attr_def_names.contains(&name.to_owned()) {
                    Some((i, name.to_owned()))
                } else {
                    None
                }
            })
            .collect();

        // Track level-to-parent_id mapping using a stack
        let mut level_stack: Vec<(String, Uuid)> = Vec::new();
        let mut objects_created: usize = 0;

        for result in rdr.records() {
            let record = result
                .map_err(|e| CoreError::BadRequest(format!("csv row error: {e}")))?;

            let level = record
                .get(level_idx)
                .unwrap_or("")
                .to_owned();

            if level.is_empty() {
                continue;
            }

            let heading = heading_idx
                .and_then(|i| record.get(i))
                .filter(|s| !s.is_empty())
                .map(String::from);

            let body = body_idx
                .and_then(|i| record.get(i))
                .filter(|s| !s.is_empty())
                .map(String::from);

            let classification = classification_idx
                .and_then(|i| record.get(i))
                .filter(|s| !s.is_empty())
                .map(String::from);

            // Build attributes JSON from matched attribute columns
            let attributes = if attr_columns.is_empty() {
                None
            } else {
                let mut map = serde_json::Map::new();
                for (idx, name) in &attr_columns {
                    if let Some(val) = record.get(*idx)
                        && !val.is_empty()
                    {
                        let _ = map.insert(
                            name.clone(),
                            serde_json::Value::String(val.to_owned()),
                        );
                    }
                }
                if map.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::Object(map))
                }
            };

            // Determine parent_id from level depth
            let depth = level.matches('.').count();
            level_stack.truncate(depth);
            let parent_id = level_stack.last().map(|(_, id)| *id);

            let input = CreateObjectInput {
                module_id,
                parent_id,
                position: None,
                heading,
                body,
                attributes,
                classification,
                references: None,
                object_type_id: None,
                lifecycle_state: None,
                lifecycle_model_id: None,
                source_object_id: None,
                source_module_id: None,
                is_placeholder: None,
            };

            let created = ObjectService::create(db, input).await?;
            level_stack.push((level, created.id));
            objects_created += 1;
        }

        Ok(CsvImportResult { objects_created })
    }
}

fn find_column(
    headers: &csv::StringRecord,
    name: &str,
) -> Result<usize, CoreError> {
    headers
        .iter()
        .position(|h| h == name)
        .ok_or_else(|| CoreError::BadRequest(format!("missing required CSV column: '{name}'")))
}
