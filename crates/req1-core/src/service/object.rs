use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
    sea_query::{Expr, Value},
};
use serde::Deserialize;
use uuid::Uuid;

use entity::{link, object, script};

use crate::PaginatedResponse;
use crate::error::CoreError;
use crate::fingerprint::compute_content_fingerprint;
use crate::history::{self, HistoryEntry};
use crate::level;
use crate::scripting::engine::{
    Mutation, ScriptEngine, ScriptLink, ScriptObject, ScriptWorld, TriggerContext,
};
use crate::suspect;
use crate::validation;

const VALID_CLASSIFICATIONS: &[&str] = &["normative", "informative", "heading"];

#[derive(Debug, Deserialize)]
pub struct CreateObjectInput {
    #[serde(default)]
    pub module_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub position: Option<i32>,
    pub heading: Option<String>,
    pub body: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub classification: Option<String>,
    pub references: Option<serde_json::Value>,
    pub object_type_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateObjectInput {
    pub parent_id: Option<Uuid>,
    pub position: Option<i32>,
    pub heading: Option<String>,
    pub body: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub reviewed: Option<bool>,
    pub classification: Option<String>,
    pub references: Option<serde_json::Value>,
    pub object_type_id: Option<Uuid>,
}

const fn default_limit() -> u64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct ListObjectsFilter {
    #[serde(default)]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub heading: Option<String>,
    pub body: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub needs_review: Option<bool>,
    pub classification: Option<String>,
    pub include_deleted: Option<bool>,
}

/// Load the `ScriptWorld` for a module (all objects + links).
pub async fn load_world(
    db: &impl ConnectionTrait,
    module_id: Uuid,
) -> Result<ScriptWorld, CoreError> {
    let module = entity::module::Entity::find_by_id(module_id)
        .one(db)
        .await?
        .ok_or_else(|| CoreError::NotFound(format!("module {module_id} not found")))?;

    let objects: Vec<ScriptObject> = object::Entity::find()
        .filter(object::Column::ModuleId.eq(module_id))
        .all(db)
        .await?
        .into_iter()
        .map(|o| ScriptObject {
            id: o.id.to_string(),
            heading: o.heading,
            body: o.body,
            level: Some(o.level),
            classification: Some(o.classification),
            attributes: o.attributes,
            version: o.current_version,
        })
        .collect();

    let object_ids: Vec<Uuid> = objects.iter().filter_map(|o| o.id.parse().ok()).collect();
    let links: Vec<ScriptLink> = if object_ids.is_empty() {
        Vec::new()
    } else {
        link::Entity::find()
            .filter(
                link::Column::SourceObjectId
                    .is_in(object_ids.clone())
                    .or(link::Column::TargetObjectId.is_in(object_ids)),
            )
            .all(db)
            .await?
            .into_iter()
            .map(|l| ScriptLink {
                id: l.id.to_string(),
                source_object_id: l.source_object_id.to_string(),
                target_object_id: l.target_object_id.to_string(),
                link_type_id: l.link_type_id.to_string(),
                suspect: l.suspect,
            })
            .collect()
    };

    Ok(ScriptWorld {
        module_id,
        module_name: module.name,
        objects,
        links,
    })
}

/// Run all enabled trigger scripts for a module + `hook_point`. Returns attribute mutations to apply.
async fn run_triggers(
    db: &impl ConnectionTrait,
    module_id: Uuid,
    hook_point: &str,
    script_obj: &ScriptObject,
) -> Result<Vec<Mutation>, CoreError> {
    let scripts = script::Entity::find()
        .filter(script::Column::ModuleId.eq(module_id))
        .filter(script::Column::ScriptType.eq("trigger"))
        .filter(script::Column::HookPoint.eq(hook_point))
        .filter(script::Column::Enabled.eq(true))
        .all(db)
        .await?;

    if scripts.is_empty() {
        return Ok(Vec::new());
    }

    let world = load_world(db, module_id).await?;
    let ctx = TriggerContext {
        hook_point: hook_point.to_owned(),
        object: script_obj.clone(),
    };

    let mut all_mutations = Vec::new();

    for s in &scripts {
        let result = ScriptEngine::run_trigger(&s.source_code, &world, &ctx)?;
        if result.rejected {
            return Err(CoreError::BadRequest(format!(
                "script '{}' rejected: {}",
                s.name,
                result
                    .reason
                    .unwrap_or_else(|| "no reason given".to_owned())
            )));
        }
        all_mutations.extend(result.mutations);
    }

    Ok(all_mutations)
}

/// Apply attribute mutations from scripts to the active model.
fn apply_mutations(
    active: &mut object::ActiveModel,
    target_id: Uuid,
    mutations: &[Mutation],
    existing_attrs: Option<&serde_json::Value>,
) {
    let mut attrs = existing_attrs
        .cloned()
        .unwrap_or(serde_json::Value::Object(serde_json::Map::default()));
    let mut changed = false;

    for m in mutations {
        match m {
            Mutation::SetAttribute {
                object_id,
                key,
                value,
            } if *object_id == target_id => {
                if let Some(obj) = attrs.as_object_mut() {
                    let _ = obj.insert(key.clone(), value.clone());
                    changed = true;
                }
            }
            Mutation::SetAttribute { .. } => {}
        }
    }

    if changed {
        active.attributes = Set(Some(attrs));
    }
}

pub struct ObjectService;

impl ObjectService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateObjectInput,
    ) -> Result<object::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        if let Some(ref attrs) = input.attributes {
            validation::validate_attributes(db, input.module_id, attrs).await?;
        }

        // Fetch module to use config defaults and required_attributes
        let module = entity::module::Entity::find_by_id(input.module_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("module {} not found", input.module_id)))?;

        let classification = input
            .classification
            .as_deref()
            .unwrap_or(module.default_classification.as_str());
        if !VALID_CLASSIFICATIONS.contains(&classification) {
            return Err(CoreError::BadRequest(format!(
                "invalid classification '{classification}', must be one of: {VALID_CLASSIFICATIONS:?}"
            )));
        }

        // Check required attributes from module config
        check_required_attributes(&module, input.attributes.as_ref())?;

        // Check object type constraints
        if let Some(type_id) = input.object_type_id {
            validation::check_object_type_constraints(db, type_id, input.attributes.as_ref())
                .await?;
        }

        // Run pre_save trigger scripts
        let script_obj = ScriptObject {
            id: id.to_string(),
            heading: input.heading.clone(),
            body: input.body.clone(),
            level: None,
            classification: Some(classification.to_owned()),
            attributes: input.attributes.clone(),
            version: 1,
        };
        let mutations = run_triggers(db, input.module_id, "pre_save", &script_obj).await?;

        // Build final attributes (base + script mutations)
        let mut final_attributes = input.attributes.clone();
        if !mutations.is_empty() {
            let mut attrs =
                final_attributes.unwrap_or(serde_json::Value::Object(serde_json::Map::default()));
            for m in &mutations {
                let Mutation::SetAttribute { key, value, .. } = m;
                if let Some(obj) = attrs.as_object_mut() {
                    let _ = obj.insert(key.clone(), value.clone());
                }
            }
            final_attributes = Some(attrs);
        }

        let fp = compute_content_fingerprint(
            input.heading.as_deref(),
            input.body.as_deref(),
            final_attributes.as_ref(),
        );

        let model = object::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            parent_id: Set(input.parent_id),
            position: Set(input.position.unwrap_or(0)),
            level: Set("0".to_owned()),
            heading: Set(input.heading.clone()),
            body: Set(input.body.clone()),
            attributes: Set(final_attributes),
            current_version: Set(1),
            classification: Set(classification.to_owned()),
            content_fingerprint: Set(fp),
            reviewed_fingerprint: Set(None),
            reviewed_at: Set(None),
            reviewed_by: Set(None),
            references_: Set(input.references.unwrap_or(serde_json::json!([]))),
            object_type_id: Set(input.object_type_id),
            deleted_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let _ = model.insert(db).await?;

        history::insert_history(
            db,
            HistoryEntry {
                object_id: id,
                module_id: input.module_id,
                version: 1,
                attribute_values: input.attributes,
                heading: input.heading,
                body: input.body,
                change_type: "create".to_owned(),
            },
        )
        .await?;

        level::recompute_module_levels(db, input.module_id).await?;

        object::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::Internal("object not found after insert".to_owned()))
    }

    #[allow(clippy::too_many_lines)]
    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateObjectInput,
    ) -> Result<object::Model, CoreError> {
        let existing = object::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("object {id} not found")))?;

        let new_version = existing.current_version + 1;
        let module_id = existing.module_id;
        let content_changed =
            input.heading.is_some() || input.body.is_some() || input.attributes.is_some();

        let mut active: object::ActiveModel = existing.clone().into();
        active.current_version = Set(new_version);
        if let Some(parent_id) = input.parent_id {
            active.parent_id = Set(Some(parent_id));
        }
        if let Some(ref heading) = input.heading {
            active.heading = Set(Some(heading.clone()));
        }
        if let Some(ref body_text) = input.body {
            active.body = Set(Some(body_text.clone()));
        }
        if let Some(ref attributes) = input.attributes {
            validation::validate_attributes(db, module_id, attributes).await?;
            active.attributes = Set(Some(attributes.clone()));
        }
        if let Some(position) = input.position {
            active.position = Set(position);
        }
        if let Some(ref classification) = input.classification {
            if !VALID_CLASSIFICATIONS.contains(&classification.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid classification '{classification}', must be one of: {VALID_CLASSIFICATIONS:?}"
                )));
            }
            active.classification = Set(classification.clone());
        }
        if let Some(ref references) = input.references {
            active.references_ = Set(references.clone());
        }
        if let Some(object_type_id) = input.object_type_id {
            active.object_type_id = Set(Some(object_type_id));
        }

        // Check required attributes: merge existing + new, then validate
        let module = entity::module::Entity::find_by_id(module_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("module {module_id} not found")))?;
        let merged_attrs = match (&existing.attributes, &input.attributes) {
            (Some(existing_a), Some(new_a)) => {
                let mut merged = existing_a.clone();
                if let (Some(base), Some(patch)) = (merged.as_object_mut(), new_a.as_object()) {
                    for (k, v) in patch {
                        let _ = base.insert(k.clone(), v.clone());
                    }
                }
                Some(merged)
            }
            (_, Some(new_a)) => Some(new_a.clone()),
            (Some(ex), None) => Some(ex.clone()),
            (None, None) => None,
        };
        check_required_attributes(&module, merged_attrs.as_ref())?;

        // Check object type constraints
        if let Some(type_id) = input.object_type_id.or(existing.object_type_id) {
            validation::check_object_type_constraints(db, type_id, merged_attrs.as_ref()).await?;
        }

        // Run pre_save trigger scripts
        let script_obj = ScriptObject {
            id: id.to_string(),
            heading: input.heading.clone().or(existing.heading.clone()),
            body: input.body.clone().or(existing.body.clone()),
            level: Some(existing.level.clone()),
            classification: Some(
                input
                    .classification
                    .clone()
                    .unwrap_or(existing.classification.clone()),
            ),
            attributes: input.attributes.clone().or(existing.attributes.clone()),
            version: new_version,
        };
        let mutations = run_triggers(db, module_id, "pre_save", &script_obj).await?;
        apply_mutations(
            &mut active,
            id,
            &mutations,
            input.attributes.as_ref().or(existing.attributes.as_ref()),
        );

        // Recompute fingerprint on content change
        if content_changed {
            let new_heading = input.heading.as_deref().or(existing.heading.as_deref());
            let new_body = input.body.as_deref().or(existing.body.as_deref());
            let new_attrs = input.attributes.as_ref().or(existing.attributes.as_ref());
            let fp = compute_content_fingerprint(new_heading, new_body, new_attrs);
            active.content_fingerprint = Set(fp.clone());

            active.reviewed_fingerprint = Set(None);
            active.reviewed_at = Set(None);
            active.reviewed_by = Set(None);

            suspect::flag_suspect_links(db, id, &fp).await?;
        } else if let Some(reviewed) = input.reviewed {
            if reviewed {
                active.reviewed_fingerprint = Set(Some(existing.content_fingerprint.clone()));
                active.reviewed_at = Set(Some(chrono::Utc::now().fixed_offset()));
            } else {
                active.reviewed_fingerprint = Set(None);
                active.reviewed_at = Set(None);
                active.reviewed_by = Set(None);
            }
        }

        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let _ = active.update(db).await?;

        history::insert_history(
            db,
            HistoryEntry {
                object_id: id,
                module_id,
                version: new_version,
                attribute_values: input.attributes,
                heading: input.heading,
                body: input.body,
                change_type: "update".to_owned(),
            },
        )
        .await?;

        if input.parent_id.is_some() || input.position.is_some() {
            level::recompute_module_levels(db, module_id).await?;
        }

        object::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::Internal("object not found after update".to_owned()))
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let existing = object::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("object {id} not found")))?;

        // Run pre_delete trigger
        let script_obj = ScriptObject {
            id: id.to_string(),
            heading: existing.heading.clone(),
            body: existing.body.clone(),
            level: Some(existing.level.clone()),
            classification: Some(existing.classification.clone()),
            attributes: existing.attributes.clone(),
            version: existing.current_version,
        };
        let _mutations = run_triggers(db, existing.module_id, "pre_delete", &script_obj).await?;

        history::insert_history(
            db,
            HistoryEntry {
                object_id: id,
                module_id: existing.module_id,
                version: existing.current_version,
                attribute_values: existing.attributes.clone(),
                heading: existing.heading.clone(),
                body: existing.body.clone(),
                change_type: "delete".to_owned(),
            },
        )
        .await?;

        let module_id = existing.module_id;
        let result = object::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("object {id} not found")));
        }

        level::recompute_module_levels(db, module_id).await?;

        Ok(())
    }

    pub async fn soft_delete(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<object::Model, CoreError> {
        let existing = object::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("object {id} not found")))?;

        let mut active: object::ActiveModel = existing.into();
        active.deleted_at = Set(Some(chrono::Utc::now().fixed_offset()));
        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<object::Model, CoreError> {
        object::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("object {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        filter: ListObjectsFilter,
    ) -> Result<PaginatedResponse<object::Model>, CoreError> {
        let mut select = object::Entity::find().filter(object::Column::ModuleId.eq(module_id));

        if filter.include_deleted != Some(true) {
            select = select.filter(object::Column::DeletedAt.is_null());
        }

        if let Some(ref heading) = filter.heading {
            select = select.filter(Expr::cust_with_values(
                "heading ILIKE $1",
                [Value::from(format!("%{heading}%"))],
            ));
        }
        if let Some(ref body) = filter.body {
            select = select.filter(Expr::cust_with_values(
                "body ILIKE $1",
                [Value::from(format!("%{body}%"))],
            ));
        }
        if let Some(ref search) = filter.search {
            select = select.filter(Expr::cust_with_values(
                "to_tsvector('english', COALESCE(heading, '') || ' ' || COALESCE(body, '')) @@ plainto_tsquery('english', $1)",
                [Value::from(search.clone())],
            ));
        }
        if filter.needs_review == Some(true) {
            select = select.filter(Expr::cust(
                "reviewed_fingerprint IS DISTINCT FROM content_fingerprint",
            ));
        }
        if let Some(ref classification) = filter.classification {
            select = select.filter(object::Column::Classification.eq(classification.clone()));
        }

        let order = match filter.sort_dir.as_deref() {
            Some("desc") => Order::Desc,
            _ => Order::Asc,
        };
        select = match filter.sort_by.as_deref() {
            Some("heading") => select.order_by(object::Column::Heading, order),
            Some("body") => select.order_by(object::Column::Body, order),
            Some("current_version") => select.order_by(object::Column::CurrentVersion, order),
            Some("updated_at") => select.order_by(object::Column::UpdatedAt, order),
            _ => select.order_by(object::Column::Level, order),
        };

        let paginator = select.paginate(db, filter.limit);
        let total = paginator.num_items().await?;
        let page = filter.offset / filter.limit;
        let items = paginator.fetch_page(page).await?;

        Ok(PaginatedResponse {
            items,
            total,
            offset: filter.offset,
            limit: filter.limit,
        })
    }
}

fn check_required_attributes(
    module: &entity::module::Model,
    attributes: Option<&serde_json::Value>,
) -> Result<(), CoreError> {
    let required: Vec<String> =
        serde_json::from_value(module.required_attributes.clone()).unwrap_or_default();
    if required.is_empty() {
        return Ok(());
    }

    let attrs_obj = attributes.and_then(|a| a.as_object());
    for attr_name in &required {
        let present = attrs_obj
            .and_then(|obj| obj.get(attr_name))
            .is_some_and(|v| !v.is_null());
        if !present {
            return Err(CoreError::BadRequest(format!(
                "missing required attribute '{attr_name}'"
            )));
        }
    }
    Ok(())
}
