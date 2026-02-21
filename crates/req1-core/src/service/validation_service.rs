use std::collections::HashSet;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use uuid::Uuid;

use entity::{link, object, script};

use crate::error::CoreError;
use crate::scripting::engine::{ScriptEngine, ScriptObject, ScriptWorld, TriggerContext};

use super::object::load_world;

/// A single validation issue found in a module.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub rule: String,
    pub severity: String,
    pub object_id: Option<String>,
    pub link_id: Option<String>,
    pub message: String,
}

/// Result of validating a module.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub module_id: String,
    pub issues: Vec<ValidationIssue>,
    pub object_count: usize,
    pub link_count: usize,
}

pub struct ValidationService;

impl ValidationService {
    /// Run all validation rules against a module and return a report.
    pub async fn validate(
        db: &impl ConnectionTrait,
        module_id: Uuid,
    ) -> Result<ValidationReport, CoreError> {
        let module = entity::module::Entity::find_by_id(module_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("module {module_id} not found")))?;

        let objects = object::Entity::find()
            .filter(object::Column::ModuleId.eq(module_id))
            .all(db)
            .await?;

        let object_ids: Vec<Uuid> = objects.iter().map(|o| o.id).collect();
        let links = if object_ids.is_empty() {
            Vec::new()
        } else {
            link::Entity::find()
                .filter(
                    link::Column::SourceObjectId
                        .is_in(object_ids.clone())
                        .or(link::Column::TargetObjectId.is_in(object_ids.clone())),
                )
                .all(db)
                .await?
        };

        let id_set: HashSet<Uuid> = object_ids.iter().copied().collect();
        let mut issues = Vec::new();

        check_objects(&objects, &id_set, &mut issues);
        check_links(&links, &id_set, &mut issues);
        check_required_attributes(&objects, &module, &mut issues);
        check_scripts(db, module_id, &objects, &mut issues).await?;

        Ok(ValidationReport {
            module_id: module_id.to_string(),
            issues,
            object_count: objects.len(),
            link_count: links.len(),
        })
    }
}

fn check_objects(
    objects: &[object::Model],
    id_set: &HashSet<Uuid>,
    issues: &mut Vec<ValidationIssue>,
) {
    for obj in objects {
        if obj.classification != "heading" && obj.heading.is_none() {
            issues.push(ValidationIssue {
                rule: "missing_heading".to_owned(),
                severity: "warning".to_owned(),
                object_id: Some(obj.id.to_string()),
                link_id: None,
                message: format!("[{}] object has no heading", obj.level),
            });
        }

        if obj.classification == "normative" && obj.body.is_none() {
            issues.push(ValidationIssue {
                rule: "missing_body".to_owned(),
                severity: "warning".to_owned(),
                object_id: Some(obj.id.to_string()),
                link_id: None,
                message: format!(
                    "[{}] {} — normative object has no body",
                    obj.level,
                    obj.heading.as_deref().unwrap_or("(no heading)")
                ),
            });
        }

        let needs_review = obj
            .reviewed_fingerprint
            .as_ref()
            .is_none_or(|fp| *fp != obj.content_fingerprint);
        if needs_review {
            issues.push(ValidationIssue {
                rule: "unreviewed".to_owned(),
                severity: "info".to_owned(),
                object_id: Some(obj.id.to_string()),
                link_id: None,
                message: format!(
                    "[{}] {} — needs review",
                    obj.level,
                    obj.heading.as_deref().unwrap_or("(no heading)")
                ),
            });
        }

        if let Some(parent_id) = obj.parent_id
            && !id_set.contains(&parent_id)
        {
            issues.push(ValidationIssue {
                rule: "orphan_object".to_owned(),
                severity: "error".to_owned(),
                object_id: Some(obj.id.to_string()),
                link_id: None,
                message: format!(
                    "[{}] {} — parent {} not found",
                    obj.level,
                    obj.heading.as_deref().unwrap_or("(no heading)"),
                    parent_id
                ),
            });
        }
    }
}

fn check_links(links: &[link::Model], id_set: &HashSet<Uuid>, issues: &mut Vec<ValidationIssue>) {
    for lnk in links {
        if lnk.suspect {
            issues.push(ValidationIssue {
                rule: "suspect_link".to_owned(),
                severity: "warning".to_owned(),
                object_id: None,
                link_id: Some(lnk.id.to_string()),
                message: format!(
                    "link {} -> {} is suspect",
                    lnk.source_object_id, lnk.target_object_id
                ),
            });
        }

        if !id_set.contains(&lnk.source_object_id) {
            issues.push(ValidationIssue {
                rule: "dangling_link".to_owned(),
                severity: "error".to_owned(),
                object_id: None,
                link_id: Some(lnk.id.to_string()),
                message: format!("link source {} not found in module", lnk.source_object_id),
            });
        }
        if !id_set.contains(&lnk.target_object_id) {
            issues.push(ValidationIssue {
                rule: "dangling_link".to_owned(),
                severity: "error".to_owned(),
                object_id: None,
                link_id: Some(lnk.id.to_string()),
                message: format!("link target {} not found in module", lnk.target_object_id),
            });
        }
    }
}

fn check_required_attributes(
    objects: &[object::Model],
    module: &entity::module::Model,
    issues: &mut Vec<ValidationIssue>,
) {
    let required: Vec<String> =
        serde_json::from_value(module.required_attributes.clone()).unwrap_or_default();
    if required.is_empty() {
        return;
    }

    for obj in objects {
        let attrs_obj = obj.attributes.as_ref().and_then(|a| a.as_object());
        for attr_name in &required {
            let present = attrs_obj
                .and_then(|a| a.get(attr_name))
                .is_some_and(|v| !v.is_null());
            if !present {
                issues.push(ValidationIssue {
                    rule: "missing_required_attribute".to_owned(),
                    severity: "error".to_owned(),
                    object_id: Some(obj.id.to_string()),
                    link_id: None,
                    message: format!(
                        "[{}] {} — missing required attribute '{attr_name}'",
                        obj.level,
                        obj.heading.as_deref().unwrap_or("(no heading)")
                    ),
                });
            }
        }
    }
}

async fn check_scripts(
    db: &impl ConnectionTrait,
    module_id: Uuid,
    objects: &[object::Model],
    issues: &mut Vec<ValidationIssue>,
) -> Result<(), CoreError> {
    let scripts = script::Entity::find()
        .filter(script::Column::ModuleId.eq(module_id))
        .filter(script::Column::ScriptType.eq("trigger"))
        .filter(script::Column::HookPoint.eq("validate"))
        .filter(script::Column::Enabled.eq(true))
        .all(db)
        .await?;

    if scripts.is_empty() {
        return Ok(());
    }

    let world = load_world(db, module_id).await?;

    for obj in objects {
        run_scripts_for_object(obj, &scripts, &world, issues);
    }

    Ok(())
}

fn run_scripts_for_object(
    obj: &object::Model,
    scripts: &[script::Model],
    world: &ScriptWorld,
    issues: &mut Vec<ValidationIssue>,
) {
    let script_obj = ScriptObject {
        id: obj.id.to_string(),
        heading: obj.heading.clone(),
        body: obj.body.clone(),
        level: Some(obj.level.clone()),
        classification: Some(obj.classification.clone()),
        attributes: obj.attributes.clone(),
        version: obj.current_version,
    };
    let ctx = TriggerContext {
        hook_point: "validate".to_owned(),
        object: script_obj,
    };

    for s in scripts {
        let result = ScriptEngine::run_trigger(&s.source_code, world, &ctx);
        match result {
            Ok(r) if r.rejected => {
                issues.push(ValidationIssue {
                    rule: format!("script:{}", s.name),
                    severity: "error".to_owned(),
                    object_id: Some(obj.id.to_string()),
                    link_id: None,
                    message: format!(
                        "[{}] {} — {}",
                        obj.level,
                        obj.heading.as_deref().unwrap_or("(no heading)"),
                        r.reason.unwrap_or_else(|| "rejected by script".to_owned())
                    ),
                });
            }
            Err(e) => {
                issues.push(ValidationIssue {
                    rule: format!("script:{}", s.name),
                    severity: "error".to_owned(),
                    object_id: Some(obj.id.to_string()),
                    link_id: None,
                    message: format!("[{}] script '{}' error: {e}", obj.level, s.name),
                });
            }
            _ => {}
        }
    }
}
