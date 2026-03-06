use std::io::Cursor;

use calamine::{Reader, Xlsx};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, Order, QueryFilter, QueryOrder};
use serde::Serialize;
use uuid::Uuid;

use entity::attribute_definition;

use crate::error::CoreError;
use crate::service::object::{CreateObjectInput, ObjectService, UpdateObjectInput};

#[derive(Debug, Serialize)]
pub struct XlsxImportResult {
    pub objects_created: usize,
    pub objects_updated: usize,
}

pub struct XlsxImportService;

impl XlsxImportService {
    #[allow(clippy::too_many_lines)]
    pub async fn import_xlsx(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        data: &[u8],
    ) -> Result<XlsxImportResult, CoreError> {
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

        let cursor = Cursor::new(data);
        let mut workbook: Xlsx<_> = Xlsx::new(cursor)
            .map_err(|e| CoreError::BadRequest(format!("invalid XLSX file: {e}")))?;

        // Find the Requirements sheet, or fall back to first sheet
        let sheet_names = workbook.sheet_names().clone();
        let sheet_name = if sheet_names.contains(&"Requirements".to_owned()) {
            "Requirements".to_owned()
        } else {
            sheet_names
                .into_iter()
                .next()
                .ok_or_else(|| CoreError::BadRequest("XLSX file has no sheets".to_owned()))?
        };

        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|e| CoreError::BadRequest(format!("cannot read sheet '{sheet_name}': {e}")))?;

        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| {
                row.iter()
                    .map(|cell| match cell {
                        calamine::Data::String(s) => s.clone(),
                        calamine::Data::Float(f) => f.to_string(),
                        calamine::Data::Int(i) => i.to_string(),
                        calamine::Data::Bool(b) => b.to_string(),
                        _ => String::new(),
                    })
                    .collect()
            })
            .collect();

        if rows.is_empty() {
            return Ok(XlsxImportResult {
                objects_created: 0,
                objects_updated: 0,
            });
        }

        // Parse headers from first row — safe: we checked `rows.is_empty()` above
        let headers = rows.first().expect("rows is non-empty");
        let id_idx = headers.iter().position(|h| h == "id");
        let level_idx = headers
            .iter()
            .position(|h| h == "level")
            .ok_or_else(|| CoreError::BadRequest("missing 'level' column in XLSX".to_owned()))?;
        let heading_idx = headers.iter().position(|h| h == "heading");
        let body_idx = headers.iter().position(|h| h == "body");
        let classification_idx = headers.iter().position(|h| h == "classification");
        let lifecycle_state_idx = headers.iter().position(|h| h == "lifecycle_state");

        // Find attribute columns
        let attr_columns: Vec<(usize, String)> = headers
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                if attr_def_names.contains(name) {
                    Some((i, name.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Fetch existing objects in module for round-trip update
        let existing_objects: std::collections::HashMap<Uuid, entity::object::Model> =
            entity::object::Entity::find()
                .filter(entity::object::Column::ModuleId.eq(module_id))
                .filter(entity::object::Column::DeletedAt.is_null())
                .all(db)
                .await?
                .into_iter()
                .map(|o| (o.id, o))
                .collect();

        let mut objects_created: usize = 0;
        let mut objects_updated: usize = 0;

        // Track level-to-parent_id mapping using a stack
        let mut level_stack: Vec<(String, Uuid)> = Vec::new();

        for row in rows.iter().skip(1) {
            let level = row.get(level_idx).map_or_else(String::new, Clone::clone);
            if level.is_empty() {
                continue;
            }

            let heading = heading_idx
                .and_then(|i| row.get(i))
                .filter(|s| !s.is_empty())
                .cloned();

            let body = body_idx
                .and_then(|i| row.get(i))
                .filter(|s| !s.is_empty())
                .cloned();

            let classification = classification_idx
                .and_then(|i| row.get(i))
                .filter(|s| !s.is_empty())
                .cloned();

            let lifecycle_state = lifecycle_state_idx
                .and_then(|i| row.get(i))
                .filter(|s| !s.is_empty())
                .cloned();

            // Build attributes JSON
            let attributes = if attr_columns.is_empty() {
                None
            } else {
                let mut map = serde_json::Map::new();
                for (idx, name) in &attr_columns {
                    if let Some(val) = row.get(*idx)
                        && !val.is_empty()
                    {
                        let _ = map.insert(
                            name.clone(),
                            serde_json::Value::String(val.clone()),
                        );
                    }
                }
                if map.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::Object(map))
                }
            };

            // Check if this is a round-trip update (id column present and matches existing)
            let existing_id = id_idx
                .and_then(|i| row.get(i))
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<Uuid>().ok());

            if let Some(obj_id) = existing_id
                && existing_objects.contains_key(&obj_id)
            {
                // Update existing object
                let update_input = UpdateObjectInput {
                    parent_id: None,
                    position: None,
                    heading,
                    body,
                    attributes,
                    reviewed: None,
                    classification,
                    references: None,
                    object_type_id: None,
                    expected_version: None,
                    lifecycle_state,
                };
                let _ = ObjectService::update(db, obj_id, update_input).await?;
                objects_updated += 1;
                level_stack.truncate(level.matches('.').count());
                level_stack.push((level, obj_id));
                continue;
            }

            // Create new object
            let depth = level.matches('.').count();
            level_stack.truncate(depth);
            let parent_id = level_stack.last().map(|(_, id)| *id);

            let create_input = CreateObjectInput {
                module_id,
                parent_id,
                position: None,
                heading,
                body,
                attributes,
                classification,
                references: None,
                object_type_id: None,
                lifecycle_state,
                lifecycle_model_id: None,
                source_object_id: None,
                source_module_id: None,
                is_placeholder: None,
            };

            let created = ObjectService::create(db, create_input).await?;
            level_stack.push((level, created.id));
            objects_created += 1;
        }

        Ok(XlsxImportResult {
            objects_created,
            objects_updated,
        })
    }
}
