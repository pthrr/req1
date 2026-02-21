use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::{attribute_definition, module, object_type, script};

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
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

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
                created_at: Set(now),
                updated_at: Set(now),
            };
            let _ = copy.insert(db).await?;
        }

        // Copy object types
        let obj_types = object_type::Entity::find()
            .filter(object_type::Column::ModuleId.eq(input.template_module_id))
            .all(db)
            .await?;
        for ot in &obj_types {
            let copy = object_type::ActiveModel {
                id: Set(Uuid::now_v7()),
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
