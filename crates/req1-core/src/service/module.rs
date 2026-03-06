use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::{attribute_definition, link, module, object, object_type, script};

use crate::PaginatedResponse;
use crate::error::CoreError;

const VALID_CLASSIFICATIONS: &[&str] = &["normative", "informative", "heading"];

#[derive(Debug, Deserialize)]
pub struct CreateModuleInput {
    pub name: String,
    #[serde(default)]
    pub project_id: Uuid,
    pub description: Option<String>,
    pub prefix: Option<String>,
    pub separator: Option<String>,
    pub digits: Option<i32>,
    pub required_attributes: Option<Vec<String>>,
    pub default_classification: Option<String>,
    pub publish_template: Option<String>,
    pub default_lifecycle_model_id: Option<Uuid>,
    pub signature_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModuleInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub prefix: Option<String>,
    pub separator: Option<String>,
    pub digits: Option<i32>,
    pub required_attributes: Option<Vec<String>>,
    pub default_classification: Option<String>,
    pub publish_template: Option<String>,
    pub default_lifecycle_model_id: Option<Uuid>,
    pub signature_config: Option<serde_json::Value>,
}

const fn default_limit() -> u64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct ListModulesFilter {
    #[serde(default)]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateModuleFromTemplateInput {
    pub name: String,
    pub project_id: Uuid,
    pub description: Option<String>,
    pub template_module_id: Uuid,
    pub copy_objects: Option<bool>,
}

pub struct ModuleService;

impl ModuleService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateModuleInput,
    ) -> Result<module::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let default_class = input
            .default_classification
            .unwrap_or_else(|| "normative".to_owned());
        if !VALID_CLASSIFICATIONS.contains(&default_class.as_str()) {
            return Err(CoreError::BadRequest(format!(
                "invalid default_classification '{default_class}', must be one of: {VALID_CLASSIFICATIONS:?}"
            )));
        }

        let model = module::ActiveModel {
            id: Set(id),
            project_id: Set(input.project_id),
            name: Set(input.name),
            description: Set(input.description),
            prefix: Set(input.prefix.unwrap_or_default()),
            separator: Set(input.separator.unwrap_or_else(|| "-".to_owned())),
            digits: Set(input.digits.unwrap_or(3)),
            required_attributes: Set(serde_json::json!(
                input.required_attributes.unwrap_or_default()
            )),
            default_classification: Set(default_class),
            publish_template: Set(input.publish_template),
            default_lifecycle_model_id: Set(input.default_lifecycle_model_id),
            signature_config: Set(input.signature_config.unwrap_or(serde_json::json!({}))),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn create_from_template(
        db: &impl ConnectionTrait,
        input: CreateModuleFromTemplateInput,
    ) -> Result<module::Model, CoreError> {
        let template = module::Entity::find_by_id(input.template_module_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                CoreError::NotFound(format!(
                    "template module {} not found",
                    input.template_module_id
                ))
            })?;

        let now = chrono::Utc::now().fixed_offset();
        let new_id = Uuid::now_v7();

        let new_module = module::ActiveModel {
            id: Set(new_id),
            project_id: Set(input.project_id),
            name: Set(input.name),
            description: Set(input.description),
            prefix: Set(template.prefix),
            separator: Set(template.separator),
            digits: Set(template.digits),
            required_attributes: Set(template.required_attributes),
            default_classification: Set(template.default_classification),
            publish_template: Set(template.publish_template),
            default_lifecycle_model_id: Set(None),
            signature_config: Set(template.signature_config),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let _ = new_module.insert(db).await?;

        // Copy attribute definitions
        let attr_defs = attribute_definition::Entity::find()
            .filter(attribute_definition::Column::ModuleId.eq(input.template_module_id))
            .all(db)
            .await?;
        for def in &attr_defs {
            let copy = attribute_definition::ActiveModel {
                id: Set(Uuid::now_v7()),
                module_id: Set(Some(new_id)),
                name: Set(def.name.clone()),
                data_type: Set(def.data_type.clone()),
                default_value: Set(def.default_value.clone()),
                enum_values: Set(def.enum_values.clone()),
                multi_select: Set(def.multi_select),
                depends_on: Set(def.depends_on),
                dependency_mapping: Set(def.dependency_mapping.clone()),
                created_at: Set(now),
            };
            let _ = copy.insert(db).await?;
        }

        // Copy scripts
        let scripts = script::Entity::find()
            .filter(script::Column::ModuleId.eq(input.template_module_id))
            .all(db)
            .await?;
        for s in &scripts {
            let copy = script::ActiveModel {
                id: Set(Uuid::now_v7()),
                module_id: Set(new_id),
                name: Set(s.name.clone()),
                script_type: Set(s.script_type.clone()),
                hook_point: Set(s.hook_point.clone()),
                source_code: Set(s.source_code.clone()),
                enabled: Set(s.enabled),
                priority: Set(s.priority),
                cron_expression: Set(None),
                last_run_at: Set(None),
                next_run_at: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            let _ = copy.insert(db).await?;
        }

        // Copy object types — build old->new ID mapping for object_type_id remapping
        let obj_types = object_type::Entity::find()
            .filter(object_type::Column::ModuleId.eq(input.template_module_id))
            .all(db)
            .await?;
        let mut type_id_map = std::collections::HashMap::new();
        for ot in &obj_types {
            let new_type_id = Uuid::now_v7();
            let _ = type_id_map.insert(ot.id, new_type_id);
            let copy = object_type::ActiveModel {
                id: Set(new_type_id),
                module_id: Set(new_id),
                name: Set(ot.name.clone()),
                description: Set(ot.description.clone()),
                default_classification: Set(ot.default_classification.clone()),
                required_attributes: Set(ot.required_attributes.clone()),
                attribute_schema: Set(ot.attribute_schema.clone()),
                created_at: Set(now),
                updated_at: Set(now),
            };
            let _ = copy.insert(db).await?;
        }

        // Copy objects + links if requested
        if input.copy_objects == Some(true) {
            let objects = object::Entity::find()
                .filter(object::Column::ModuleId.eq(input.template_module_id))
                .filter(object::Column::DeletedAt.is_null())
                .order_by(object::Column::Position, Order::Asc)
                .all(db)
                .await?;

            // Build old_id -> new_id mapping
            let mut obj_id_map = std::collections::HashMap::new();
            for obj in &objects {
                let _ = obj_id_map.insert(obj.id, Uuid::now_v7());
            }

            // Insert copied objects with remapped parent_id and object_type_id
            for obj in &objects {
                let new_obj_id = *obj_id_map.get(&obj.id).ok_or_else(|| {
                    CoreError::Internal("object id not in mapping".to_owned())
                })?;
                let new_parent_id = obj.parent_id.and_then(|pid| obj_id_map.get(&pid).copied());
                let new_type_id = obj.object_type_id.and_then(|tid| type_id_map.get(&tid).copied());
                let copy = object::ActiveModel {
                    id: Set(new_obj_id),
                    module_id: Set(new_id),
                    parent_id: Set(new_parent_id),
                    position: Set(obj.position),
                    level: Set(obj.level.clone()),
                    heading: Set(obj.heading.clone()),
                    body: Set(obj.body.clone()),
                    attributes: Set(obj.attributes.clone()),
                    current_version: Set(1),
                    classification: Set(obj.classification.clone()),
                    content_fingerprint: Set(obj.content_fingerprint.clone()),
                    reviewed_fingerprint: Set(None),
                    reviewed_at: Set(None),
                    reviewed_by: Set(None),
                    references_: Set(obj.references_.clone()),
                    object_type_id: Set(new_type_id),
                    lifecycle_state: Set(obj.lifecycle_state.clone()),
                    lifecycle_model_id: Set(None),
                    source_object_id: Set(None),
                    source_module_id: Set(None),
                    is_placeholder: Set(false),
                    docx_source_id: Set(None),
                    deleted_at: Set(None),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                let _ = copy.insert(db).await?;
            }

            // Copy links where both source and target are in the copied objects
            let old_ids: Vec<Uuid> = obj_id_map.keys().copied().collect();
            if !old_ids.is_empty() {
                let links = link::Entity::find()
                    .filter(link::Column::SourceObjectId.is_in(old_ids.clone()))
                    .filter(link::Column::TargetObjectId.is_in(old_ids))
                    .all(db)
                    .await?;

                for lnk in &links {
                    if let (Some(new_src), Some(new_tgt)) = (
                        obj_id_map.get(&lnk.source_object_id),
                        obj_id_map.get(&lnk.target_object_id),
                    ) {
                        let copy = link::ActiveModel {
                            id: Set(Uuid::now_v7()),
                            source_object_id: Set(*new_src),
                            target_object_id: Set(*new_tgt),
                            link_type_id: Set(lnk.link_type_id),
                            attributes: Set(lnk.attributes.clone()),
                            suspect: Set(false),
                            source_fingerprint: Set(lnk.source_fingerprint.clone()),
                            target_fingerprint: Set(lnk.target_fingerprint.clone()),
                            created_at: Set(now),
                            updated_at: Set(now),
                        };
                        let _ = copy.insert(db).await?;
                    }
                }
            }

            // Recompute levels on the new module
            crate::level::recompute_module_levels(db, new_id).await?;
        }

        module::Entity::find_by_id(new_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::Internal("module not found after insert".to_owned()))
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateModuleInput,
    ) -> Result<module::Model, CoreError> {
        let existing = module::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("module {id} not found")))?;

        let mut active: module::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(prefix) = input.prefix {
            active.prefix = Set(prefix);
        }
        if let Some(separator) = input.separator {
            active.separator = Set(separator);
        }
        if let Some(digits) = input.digits {
            active.digits = Set(digits);
        }
        if let Some(required_attributes) = input.required_attributes {
            active.required_attributes = Set(serde_json::json!(required_attributes));
        }
        if let Some(ref default_classification) = input.default_classification {
            if !VALID_CLASSIFICATIONS.contains(&default_classification.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid default_classification '{default_classification}', must be one of: {VALID_CLASSIFICATIONS:?}"
                )));
            }
            active.default_classification = Set(default_classification.clone());
        }
        if let Some(publish_template) = input.publish_template {
            active.publish_template = Set(Some(publish_template));
        }
        if let Some(lifecycle_id) = input.default_lifecycle_model_id {
            active.default_lifecycle_model_id = Set(Some(lifecycle_id));
        }
        if let Some(signature_config) = input.signature_config {
            active.signature_config = Set(signature_config);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = module::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("module {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<module::Model, CoreError> {
        module::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("module {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        filter: ListModulesFilter,
    ) -> Result<PaginatedResponse<module::Model>, CoreError> {
        let mut select = module::Entity::find();
        if let Some(project_id) = filter.project_id {
            select = select.filter(module::Column::ProjectId.eq(project_id));
        }
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
