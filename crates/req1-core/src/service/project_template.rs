use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entity::{
    attribute_definition, lifecycle_model, module, object, object_type, project, project_template,
};

use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateTemplateInput {
    pub name: String,
    pub description: Option<String>,
    pub standard: Option<String>,
    pub version: Option<String>,
    pub template_data: serde_json::Value,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub standard: Option<String>,
    pub version: Option<String>,
    pub template_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct InstantiateInput {
    pub workspace_id: Uuid,
    pub project_name: String,
    pub project_description: Option<String>,
    pub include_seed_objects: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct InstantiateResult {
    pub project_id: Uuid,
    pub modules_created: usize,
}

pub struct ProjectTemplateService;

impl ProjectTemplateService {
    pub async fn list(
        db: &impl ConnectionTrait,
    ) -> Result<Vec<project_template::Model>, CoreError> {
        let templates = project_template::Entity::find().all(db).await?;
        Ok(templates)
    }

    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateTemplateInput,
    ) -> Result<project_template::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = project_template::ActiveModel {
            id: Set(id),
            name: Set(input.name),
            description: Set(input.description),
            standard: Set(input.standard),
            version: Set(input.version),
            template_data: Set(input.template_data),
            is_builtin: Set(false),
            created_by: Set(input.created_by),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<project_template::Model, CoreError> {
        project_template::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("project template {id} not found")))
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateTemplateInput,
    ) -> Result<project_template::Model, CoreError> {
        let existing = project_template::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("project template {id} not found")))?;

        let mut active: project_template::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(standard) = input.standard {
            active.standard = Set(Some(standard));
        }
        if let Some(version) = input.version {
            active.version = Set(Some(version));
        }
        if let Some(template_data) = input.template_data {
            active.template_data = Set(template_data);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let existing = project_template::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("project template {id} not found")))?;

        if existing.is_builtin {
            return Err(CoreError::BadRequest(
                "cannot delete a built-in template".to_owned(),
            ));
        }

        let result = project_template::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "project template {id} not found"
            )));
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub async fn instantiate(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: InstantiateInput,
    ) -> Result<InstantiateResult, CoreError> {
        let template = project_template::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("project template {id} not found")))?;

        let now = chrono::Utc::now().fixed_offset();
        let project_id = Uuid::now_v7();

        let project_model = project::ActiveModel {
            id: Set(project_id),
            workspace_id: Set(input.workspace_id),
            name: Set(input.project_name),
            description: Set(input.project_description),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let _ = project_model.insert(db).await?;

        let data = &template.template_data;
        let modules_arr = data
            .get("modules")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let include_seed = input.include_seed_objects.unwrap_or(false);
        let mut modules_created: usize = 0;

        for module_def in &modules_arr {
            let module_id = Uuid::now_v7();
            let module_name = module_def
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_owned();
            let prefix = module_def
                .get("prefix")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();
            let separator = module_def
                .get("separator")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_owned();
            let digits = module_def
                .get("digits")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or(3);
            let default_classification = module_def
                .get("default_classification")
                .and_then(|v| v.as_str())
                .unwrap_or("normative")
                .to_owned();

            let new_module = module::ActiveModel {
                id: Set(module_id),
                project_id: Set(project_id),
                name: Set(module_name),
                description: Set(None),
                prefix: Set(prefix),
                separator: Set(separator),
                digits: Set(digits),
                required_attributes: Set(serde_json::json!([])),
                default_classification: Set(default_classification),
                publish_template: Set(None),
                default_lifecycle_model_id: Set(None),
                signature_config: Set(serde_json::json!({})),
                created_at: Set(now),
                updated_at: Set(now),
            };
            let _ = new_module.insert(db).await?;

            // Create attribute definitions
            let attr_defs = module_def
                .get("attribute_definitions")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            for attr_def in &attr_defs {
                let attr_name = attr_def
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unnamed")
                    .to_owned();
                let data_type = attr_def
                    .get("data_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("string")
                    .to_owned();
                let enum_values = attr_def.get("enum_values").cloned();

                let ad = attribute_definition::ActiveModel {
                    id: Set(Uuid::now_v7()),
                    module_id: Set(Some(module_id)),
                    name: Set(attr_name),
                    data_type: Set(data_type),
                    default_value: Set(None),
                    enum_values: Set(enum_values),
                    multi_select: Set(false),
                    depends_on: Set(None),
                    dependency_mapping: Set(None),
                    created_at: Set(now),
                };
                let _ = ad.insert(db).await?;
            }

            // Create object types
            let obj_types = module_def
                .get("object_types")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            for ot_def in &obj_types {
                let ot_name = ot_def
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unnamed")
                    .to_owned();
                let ot_class = ot_def
                    .get("default_classification")
                    .and_then(|v| v.as_str())
                    .unwrap_or("normative")
                    .to_owned();
                let required_attrs = ot_def
                    .get("required_attributes")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));

                let ot = object_type::ActiveModel {
                    id: Set(Uuid::now_v7()),
                    module_id: Set(module_id),
                    name: Set(ot_name),
                    description: Set(None),
                    default_classification: Set(ot_class),
                    required_attributes: Set(required_attrs),
                    attribute_schema: Set(serde_json::json!({})),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                let _ = ot.insert(db).await?;
            }

            // Create lifecycle model
            if let Some(lc_def) = module_def.get("lifecycle_model") {
                let lc_name = lc_def
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Default Lifecycle")
                    .to_owned();
                let initial_state = lc_def
                    .get("initial_state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("draft")
                    .to_owned();
                let states = lc_def
                    .get("states")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));
                let transitions = lc_def
                    .get("transitions")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));

                let lc_id = Uuid::now_v7();
                let lc = lifecycle_model::ActiveModel {
                    id: Set(lc_id),
                    module_id: Set(module_id),
                    name: Set(lc_name),
                    description: Set(None),
                    initial_state: Set(initial_state),
                    states: Set(states),
                    transitions: Set(transitions),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                let _ = lc.insert(db).await?;
            }

            // Create seed objects
            if include_seed {
                let seed_objects = module_def
                    .get("seed_objects")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                for (position, seed) in (0_i32..).zip(seed_objects.iter()) {
                    let heading = seed
                        .get("heading")
                        .and_then(|v| v.as_str())
                        .map(str::to_owned);
                    let classification = seed
                        .get("classification")
                        .and_then(|v| v.as_str())
                        .unwrap_or("heading")
                        .to_owned();

                    let parent_id = Uuid::now_v7();
                    let parent_obj = object::ActiveModel {
                        id: Set(parent_id),
                        module_id: Set(module_id),
                        parent_id: Set(None),
                        position: Set(position),
                        level: Set("0".to_owned()),
                        heading: Set(heading),
                        body: Set(None),
                        attributes: Set(None),
                        current_version: Set(1),
                        classification: Set(classification),
                        content_fingerprint: Set(String::new()),
                        reviewed_fingerprint: Set(None),
                        reviewed_at: Set(None),
                        reviewed_by: Set(None),
                        references_: Set(serde_json::json!([])),
                        object_type_id: Set(None),
                        lifecycle_state: Set(None),
                        lifecycle_model_id: Set(None),
                        source_object_id: Set(None),
                        source_module_id: Set(None),
                        is_placeholder: Set(false),
                        docx_source_id: Set(None),
                        deleted_at: Set(None),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    let _ = parent_obj.insert(db).await?;

                    let children = seed
                        .get("children")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();

                    for (child_position, child) in (0_i32..).zip(children.iter()) {
                        let child_heading = child
                            .get("heading")
                            .and_then(|v| v.as_str())
                            .map(str::to_owned);
                        let child_body = child
                            .get("body")
                            .and_then(|v| v.as_str())
                            .map(str::to_owned);
                        let child_class = child
                            .get("classification")
                            .and_then(|v| v.as_str())
                            .unwrap_or("normative")
                            .to_owned();

                        let child_obj = object::ActiveModel {
                            id: Set(Uuid::now_v7()),
                            module_id: Set(module_id),
                            parent_id: Set(Some(parent_id)),
                            position: Set(child_position),
                            level: Set("0".to_owned()),
                            heading: Set(child_heading),
                            body: Set(child_body),
                            attributes: Set(None),
                            current_version: Set(1),
                            classification: Set(child_class),
                            content_fingerprint: Set(String::new()),
                            reviewed_fingerprint: Set(None),
                            reviewed_at: Set(None),
                            reviewed_by: Set(None),
                            references_: Set(serde_json::json!([])),
                            object_type_id: Set(None),
                            lifecycle_state: Set(None),
                            lifecycle_model_id: Set(None),
                            source_object_id: Set(None),
                            source_module_id: Set(None),
                            is_placeholder: Set(false),
                            docx_source_id: Set(None),
                            deleted_at: Set(None),
                            created_at: Set(now),
                            updated_at: Set(now),
                        };
                        let _ = child_obj.insert(db).await?;
                    }
                }
            }

            modules_created += 1;
        }

        Ok(InstantiateResult {
            project_id,
            modules_created,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn seed_builtins(db: &impl ConnectionTrait) -> Result<(), CoreError> {
        let now = chrono::Utc::now().fixed_offset();

        // ISO 26262 - Automotive Functional Safety
        let existing_iso = project_template::Entity::find()
            .filter(project_template::Column::Name.eq("ISO 26262"))
            .one(db)
            .await?;
        if existing_iso.is_none() {
            let iso_data = serde_json::json!({
                "modules": [
                    {
                        "name": "Safety Requirements Specification",
                        "prefix": "SRS",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "ASIL",
                                "data_type": "enum",
                                "enum_values": ["QM", "A", "B", "C", "D"]
                            },
                            {
                                "name": "Verification Method",
                                "data_type": "enum",
                                "enum_values": ["Review", "Analysis", "Simulation", "Test"]
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Safety Goal",
                                "default_classification": "normative",
                                "required_attributes": ["ASIL"]
                            },
                            {
                                "name": "Functional Safety Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["ASIL", "Verification Method"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "ISO 26262 Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "in_review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "in_review"},
                                {"from": "in_review", "to": "draft"},
                                {"from": "in_review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Safety Goals",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SG-001",
                                        "body": "The system shall prevent unintended acceleration.",
                                        "classification": "normative"
                                    }
                                ]
                            },
                            {
                                "heading": "Functional Safety Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "FSR-001",
                                        "body": "The system shall detect sensor failures within 10ms.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Technical Safety Concept",
                        "prefix": "TSC",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "ASIL",
                                "data_type": "enum",
                                "enum_values": ["QM", "A", "B", "C", "D"]
                            },
                            {
                                "name": "Verification Method",
                                "data_type": "enum",
                                "enum_values": ["Review", "Analysis", "Simulation", "Test"]
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Technical Safety Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["ASIL"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "ISO 26262 Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "in_review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "in_review"},
                                {"from": "in_review", "to": "draft"},
                                {"from": "in_review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Technical Safety Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "TSR-001",
                                        "body": "The microcontroller shall implement hardware watchdog with 5ms timeout.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Hardware Safety Requirements",
                        "prefix": "HWSR",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "ASIL",
                                "data_type": "enum",
                                "enum_values": ["QM", "A", "B", "C", "D"]
                            },
                            {
                                "name": "Verification Method",
                                "data_type": "enum",
                                "enum_values": ["Review", "Analysis", "Simulation", "Test"]
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Hardware Safety Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["ASIL"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "ISO 26262 Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "in_review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "in_review"},
                                {"from": "in_review", "to": "draft"},
                                {"from": "in_review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Hardware Safety Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "HWSR-001",
                                        "body": "The power supply shall provide brownout detection with automatic shutdown.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Software Safety Requirements",
                        "prefix": "SWSR",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "ASIL",
                                "data_type": "enum",
                                "enum_values": ["QM", "A", "B", "C", "D"]
                            },
                            {
                                "name": "Verification Method",
                                "data_type": "enum",
                                "enum_values": ["Review", "Analysis", "Simulation", "Test"]
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Software Safety Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["ASIL"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "ISO 26262 Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "in_review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "in_review"},
                                {"from": "in_review", "to": "draft"},
                                {"from": "in_review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Software Safety Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SWSR-001",
                                        "body": "The software shall perform a RAM test at startup.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Safety Validation Plan",
                        "prefix": "SVP",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "ASIL",
                                "data_type": "enum",
                                "enum_values": ["QM", "A", "B", "C", "D"]
                            },
                            {
                                "name": "Verification Method",
                                "data_type": "enum",
                                "enum_values": ["Review", "Analysis", "Simulation", "Test"]
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Validation Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["Verification Method"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "ISO 26262 Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "in_review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "in_review"},
                                {"from": "in_review", "to": "draft"},
                                {"from": "in_review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Validation Activities",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "VAL-001",
                                        "body": "Validate that all safety goals are addressed by the safety concept.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    }
                ],
                "link_types": [
                    {"name": "Satisfies", "description": "Lower-level requirement satisfies higher-level requirement"},
                    {"name": "Verifies", "description": "Test case verifies a requirement"},
                    {"name": "Derives From", "description": "Requirement is derived from a parent requirement"}
                ]
            });

            let iso_template = project_template::ActiveModel {
                id: Set(Uuid::now_v7()),
                name: Set("ISO 26262".to_owned()),
                description: Set(Some(
                    "Automotive functional safety standard template with ASIL classification and full traceability structure."
                        .to_owned(),
                )),
                standard: Set(Some("ISO 26262".to_owned())),
                version: Set(Some("2018".to_owned())),
                template_data: Set(iso_data),
                is_builtin: Set(true),
                created_by: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            let _ = iso_template.insert(db).await?;
        }

        // DO-178C - Airborne Systems Software
        let existing_do178 = project_template::Entity::find()
            .filter(project_template::Column::Name.eq("DO-178C"))
            .one(db)
            .await?;
        if existing_do178.is_none() {
            let do178_data = serde_json::json!({
                "modules": [
                    {
                        "name": "System Requirements",
                        "prefix": "SYSREQ",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "DAL",
                                "data_type": "enum",
                                "enum_values": ["A", "B", "C", "D", "E"]
                            },
                            {
                                "name": "Software Level",
                                "data_type": "enum",
                                "enum_values": ["Level A", "Level B", "Level C", "Level D", "Level E"]
                            },
                            {
                                "name": "Derived Requirement",
                                "data_type": "boolean",
                                "enum_values": null
                            }
                        ],
                        "object_types": [
                            {
                                "name": "System Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["DAL"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "DO-178C Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "review"},
                                {"from": "review", "to": "draft"},
                                {"from": "review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "System Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SYSREQ-001",
                                        "body": "The system shall provide continuous monitoring of flight-critical parameters.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "High-Level Requirements",
                        "prefix": "HLR",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "DAL",
                                "data_type": "enum",
                                "enum_values": ["A", "B", "C", "D", "E"]
                            },
                            {
                                "name": "Software Level",
                                "data_type": "enum",
                                "enum_values": ["Level A", "Level B", "Level C", "Level D", "Level E"]
                            },
                            {
                                "name": "Derived Requirement",
                                "data_type": "boolean",
                                "enum_values": null
                            }
                        ],
                        "object_types": [
                            {
                                "name": "High-Level Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["DAL"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "DO-178C Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "review"},
                                {"from": "review", "to": "draft"},
                                {"from": "review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "High-Level Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "HLR-001",
                                        "body": "The software shall process sensor input data at a minimum rate of 50Hz.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Low-Level Requirements",
                        "prefix": "LLR",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "DAL",
                                "data_type": "enum",
                                "enum_values": ["A", "B", "C", "D", "E"]
                            },
                            {
                                "name": "Software Level",
                                "data_type": "enum",
                                "enum_values": ["Level A", "Level B", "Level C", "Level D", "Level E"]
                            },
                            {
                                "name": "Derived Requirement",
                                "data_type": "boolean",
                                "enum_values": null
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Low-Level Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["DAL"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "DO-178C Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "review"},
                                {"from": "review", "to": "draft"},
                                {"from": "review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Low-Level Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "LLR-001",
                                        "body": "The ADC read function shall return a 12-bit unsigned integer within 2ms.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Software Design",
                        "prefix": "SDD",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "DAL",
                                "data_type": "enum",
                                "enum_values": ["A", "B", "C", "D", "E"]
                            },
                            {
                                "name": "Software Level",
                                "data_type": "enum",
                                "enum_values": ["Level A", "Level B", "Level C", "Level D", "Level E"]
                            },
                            {
                                "name": "Derived Requirement",
                                "data_type": "boolean",
                                "enum_values": null
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Design Element",
                                "default_classification": "normative",
                                "required_attributes": []
                            }
                        ],
                        "lifecycle_model": {
                            "name": "DO-178C Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "review"},
                                {"from": "review", "to": "draft"},
                                {"from": "review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Software Architecture",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SDD-001",
                                        "body": "The system shall use a layered architecture with separation between application and BSP layers.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Test Cases",
                        "prefix": "TC",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "DAL",
                                "data_type": "enum",
                                "enum_values": ["A", "B", "C", "D", "E"]
                            },
                            {
                                "name": "Software Level",
                                "data_type": "enum",
                                "enum_values": ["Level A", "Level B", "Level C", "Level D", "Level E"]
                            },
                            {
                                "name": "Derived Requirement",
                                "data_type": "boolean",
                                "enum_values": null
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Test Case",
                                "default_classification": "normative",
                                "required_attributes": ["DAL"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "DO-178C Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "review"},
                                {"from": "review", "to": "draft"},
                                {"from": "review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Test Procedures",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "TC-001",
                                        "body": "Verify that sensor input processing meets the 50Hz minimum rate requirement.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    }
                ],
                "link_types": [
                    {"name": "Satisfies", "description": "Lower-level requirement satisfies higher-level requirement"},
                    {"name": "Verifies", "description": "Test case verifies a requirement"},
                    {"name": "Derives From", "description": "Derived requirement traces to its source"}
                ]
            });

            let do178_template = project_template::ActiveModel {
                id: Set(Uuid::now_v7()),
                name: Set("DO-178C".to_owned()),
                description: Set(Some(
                    "Airborne systems software standard template with DAL classification levels A through E."
                        .to_owned(),
                )),
                standard: Set(Some("DO-178C".to_owned())),
                version: Set(Some("2011".to_owned())),
                template_data: Set(do178_data),
                is_builtin: Set(true),
                created_by: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            let _ = do178_template.insert(db).await?;
        }

        // IEC 62304 - Medical Device Software
        let existing_iec = project_template::Entity::find()
            .filter(project_template::Column::Name.eq("IEC 62304"))
            .one(db)
            .await?;
        if existing_iec.is_none() {
            let iec_data = serde_json::json!({
                "modules": [
                    {
                        "name": "Software Requirements Specification",
                        "prefix": "SWRS",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "Safety Class",
                                "data_type": "enum",
                                "enum_values": ["A", "B", "C"]
                            },
                            {
                                "name": "Risk Control Measure",
                                "data_type": "string",
                                "enum_values": null
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Software Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["Safety Class"]
                            },
                            {
                                "name": "Risk Control Requirement",
                                "default_classification": "normative",
                                "required_attributes": ["Safety Class", "Risk Control Measure"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "IEC 62304 Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "review"},
                                {"from": "review", "to": "draft"},
                                {"from": "review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Functional Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SWRS-001",
                                        "body": "The software shall display patient vital signs with a refresh rate of at least 1Hz.",
                                        "classification": "normative"
                                    }
                                ]
                            },
                            {
                                "heading": "Performance Requirements",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SWRS-002",
                                        "body": "The software shall respond to alarm conditions within 250ms.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Software Architecture",
                        "prefix": "SWARCH",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "Safety Class",
                                "data_type": "enum",
                                "enum_values": ["A", "B", "C"]
                            },
                            {
                                "name": "Risk Control Measure",
                                "data_type": "string",
                                "enum_values": null
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Architecture Element",
                                "default_classification": "normative",
                                "required_attributes": ["Safety Class"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "IEC 62304 Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "review"},
                                {"from": "review", "to": "draft"},
                                {"from": "review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Software Items",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SWARCH-001",
                                        "body": "The system shall be decomposed into software items with defined interfaces.",
                                        "classification": "normative"
                                    }
                                ]
                            },
                            {
                                "heading": "Segregation",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SWARCH-002",
                                        "body": "Safety-critical software items shall be segregated from non-safety-critical items.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "Detailed Design",
                        "prefix": "SWDD",
                        "separator": "-",
                        "digits": 4,
                        "default_classification": "normative",
                        "attribute_definitions": [
                            {
                                "name": "Safety Class",
                                "data_type": "enum",
                                "enum_values": ["A", "B", "C"]
                            },
                            {
                                "name": "Risk Control Measure",
                                "data_type": "string",
                                "enum_values": null
                            }
                        ],
                        "object_types": [
                            {
                                "name": "Design Element",
                                "default_classification": "normative",
                                "required_attributes": ["Safety Class"]
                            }
                        ],
                        "lifecycle_model": {
                            "name": "IEC 62304 Lifecycle",
                            "initial_state": "draft",
                            "states": [
                                {"name": "draft", "color": "#6B7280"},
                                {"name": "review", "color": "#F59E0B"},
                                {"name": "approved", "color": "#10B981"},
                                {"name": "released", "color": "#3B82F6"}
                            ],
                            "transitions": [
                                {"from": "draft", "to": "review"},
                                {"from": "review", "to": "draft"},
                                {"from": "review", "to": "approved"},
                                {"from": "approved", "to": "released"},
                                {"from": "released", "to": "draft"}
                            ]
                        },
                        "seed_objects": [
                            {
                                "heading": "Software Unit Design",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SWDD-001",
                                        "body": "Each software unit shall have a defined interface specification and error handling strategy.",
                                        "classification": "normative"
                                    }
                                ]
                            },
                            {
                                "heading": "Algorithm Design",
                                "classification": "heading",
                                "children": [
                                    {
                                        "heading": "SWDD-002",
                                        "body": "All algorithms shall be documented with input ranges, output ranges, and precision requirements.",
                                        "classification": "normative"
                                    }
                                ]
                            }
                        ]
                    }
                ],
                "link_types": [
                    {"name": "Satisfies", "description": "Lower-level requirement satisfies higher-level requirement"},
                    {"name": "Implements", "description": "Design element implements a requirement"},
                    {"name": "Mitigates", "description": "Requirement mitigates an identified risk"}
                ]
            });

            let iec_template = project_template::ActiveModel {
                id: Set(Uuid::now_v7()),
                name: Set("IEC 62304".to_owned()),
                description: Set(Some(
                    "Medical device software lifecycle standard template with Safety Class A/B/C classification."
                        .to_owned(),
                )),
                standard: Set(Some("IEC 62304".to_owned())),
                version: Set(Some("2015".to_owned())),
                template_data: Set(iec_data),
                is_builtin: Set(true),
                created_by: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            let _ = iec_template.insert(db).await?;
        }

        Ok(())
    }
}
