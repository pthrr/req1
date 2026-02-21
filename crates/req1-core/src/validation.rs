use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use entity::attribute_definition;

use crate::error::CoreError;

pub async fn validate_attributes(
    db: &impl ConnectionTrait,
    module_id: Uuid,
    attributes: &serde_json::Value,
) -> Result<(), CoreError> {
    let obj = attributes
        .as_object()
        .ok_or_else(|| CoreError::BadRequest("attributes must be a JSON object".to_owned()))?;

    if obj.is_empty() {
        return Ok(());
    }

    let defs = attribute_definition::Entity::find()
        .filter(
            attribute_definition::Column::ModuleId
                .eq(module_id)
                .or(attribute_definition::Column::ModuleId.is_null()),
        )
        .all(db)
        .await?;

    let def_map: std::collections::HashMap<&str, &attribute_definition::Model> =
        defs.iter().map(|d| (d.name.as_str(), d)).collect();

    for (key, value) in obj {
        let def = def_map
            .get(key.as_str())
            .ok_or_else(|| CoreError::BadRequest(format!("unknown attribute '{key}'")))?;

        if value.is_null() {
            continue;
        }

        match def.data_type.as_str() {
            "string" | "rich_text" | "user_ref" | "date" => {
                if !value.is_string() {
                    return Err(CoreError::BadRequest(format!(
                        "attribute '{key}' must be a string"
                    )));
                }
            }
            "integer" => {
                if !value.is_i64() && !value.is_u64() {
                    return Err(CoreError::BadRequest(format!(
                        "attribute '{key}' must be an integer"
                    )));
                }
            }
            "float" => {
                if !value.is_number() {
                    return Err(CoreError::BadRequest(format!(
                        "attribute '{key}' must be a number"
                    )));
                }
            }
            "bool" => {
                if !value.is_boolean() {
                    return Err(CoreError::BadRequest(format!(
                        "attribute '{key}' must be a boolean"
                    )));
                }
            }
            "enum" => {
                let val_str = value.as_str().ok_or_else(|| {
                    CoreError::BadRequest(format!("attribute '{key}' must be a string"))
                })?;
                if let Some(ref ev) = def.enum_values {
                    let allowed: Vec<&str> = ev
                        .as_array()
                        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                        .unwrap_or_default();
                    if !allowed.contains(&val_str) {
                        return Err(CoreError::BadRequest(format!(
                            "attribute '{key}' value '{val_str}' not in allowed values: {allowed:?}"
                        )));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub async fn check_object_type_constraints(
    db: &impl ConnectionTrait,
    object_type_id: Uuid,
    attributes: Option<&serde_json::Value>,
) -> Result<(), CoreError> {
    let object_type = entity::object_type::Entity::find_by_id(object_type_id)
        .one(db)
        .await?
        .ok_or_else(|| {
            CoreError::BadRequest(format!("object type '{object_type_id}' not found"))
        })?;

    // Check required attributes from the object type
    let required: Vec<String> =
        serde_json::from_value(object_type.required_attributes.clone()).unwrap_or_default();
    if !required.is_empty() {
        let attrs_obj = attributes.and_then(|a| a.as_object());
        for attr_name in &required {
            let present = attrs_obj
                .and_then(|obj| obj.get(attr_name))
                .is_some_and(|v| !v.is_null());
            if !present {
                return Err(CoreError::BadRequest(format!(
                    "object type '{}' requires attribute '{attr_name}'",
                    object_type.name
                )));
            }
        }
    }

    // Validate attribute types against the object type's attribute_schema
    if let Some(schema_obj) = object_type.attribute_schema.as_object()
        && let Some(attrs) = attributes.and_then(|a| a.as_object())
    {
        for (key, value) in attrs {
            if value.is_null() {
                continue;
            }
            if let Some(schema_entry) = schema_obj.get(key).and_then(|s| s.as_object())
                && let Some(expected_type) = schema_entry.get("type").and_then(|t| t.as_str())
            {
                let valid = match expected_type {
                    "string" => value.is_string(),
                    "integer" => value.is_i64() || value.is_u64(),
                    "float" | "number" => value.is_number(),
                    "bool" | "boolean" => value.is_boolean(),
                    _ => true,
                };
                if !valid {
                    return Err(CoreError::BadRequest(format!(
                        "attribute '{key}' must be of type '{expected_type}' per object type '{}'",
                        object_type.name
                    )));
                }
            }
        }
    }

    Ok(())
}

pub fn validate_attr_constraints(
    data_type: &str,
    default_value: &Option<String>,
    enum_values: &Option<serde_json::Value>,
) -> Result<(), CoreError> {
    if data_type == "enum" {
        match enum_values {
            Some(v) if v.as_array().is_some_and(|a| !a.is_empty()) => {}
            _ => {
                return Err(CoreError::BadRequest(
                    "enum type requires a non-empty enum_values array".to_owned(),
                ));
            }
        }
    } else if enum_values.is_some() {
        return Err(CoreError::BadRequest(format!(
            "enum_values may only be set when data_type is 'enum', not '{data_type}'"
        )));
    }

    if let Some(val) = default_value {
        match data_type {
            "integer" => {
                if val.parse::<i64>().is_err() {
                    return Err(CoreError::BadRequest(format!(
                        "default_value '{val}' is not a valid integer"
                    )));
                }
            }
            "float" => {
                if val.parse::<f64>().is_err() {
                    return Err(CoreError::BadRequest(format!(
                        "default_value '{val}' is not a valid float"
                    )));
                }
            }
            "bool" => {
                if val != "true" && val != "false" {
                    return Err(CoreError::BadRequest(format!(
                        "default_value '{val}' is not a valid bool (expected 'true' or 'false')"
                    )));
                }
            }
            "enum" => {
                if let Some(ev) = enum_values {
                    let allowed: Vec<&str> = ev
                        .as_array()
                        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                        .unwrap_or_default();
                    if !allowed.contains(&val.as_str()) {
                        return Err(CoreError::BadRequest(format!(
                            "default_value '{val}' is not in enum_values: {allowed:?}"
                        )));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
