use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ConnectionTrait, Set, TransactionTrait};
use serde_json::json;
use uuid::Uuid;

use entity::{attribute_definition, link, link_type, module, object, object_type};
use req1_reqif::{
    AttributeDefinition as ReqifAttrDef, DatatypeDefinition, ReqIf, SpecHierarchyChildren, SpecType,
};

use crate::error::CoreError;
use crate::fingerprint::compute_content_fingerprint;

use super::ImportResult;
use super::type_map::{
    attr_value_def_ref, datatype_identifier, enum_values_to_json, extract_enum_values,
    reqif_attr_value_to_json, reqif_datatype_to_entity,
};

/// Import a parsed `ReqIF` document into the database, creating entities for a given project.
///
/// Runs in a single transaction; on failure everything is rolled back.
#[allow(clippy::too_many_lines)]
pub async fn import_reqif(
    db: &(impl ConnectionTrait + TransactionTrait),
    project_id: Uuid,
    doc: &ReqIf,
) -> Result<ImportResult, CoreError> {
    let content = &doc.core_content.req_if_content;

    // Validate: at least one specification
    let specifications = content
        .specifications
        .as_ref()
        .and_then(|s| {
            if s.specifications.is_empty() {
                None
            } else {
                Some(&s.specifications)
            }
        })
        .ok_or_else(|| CoreError::bad_request("ReqIF document has no specifications".to_owned()))?;

    let txn = db.begin().await?;

    // Build lookup maps from ReqIF elements
    let datatypes: &[DatatypeDefinition] = content
        .datatypes
        .as_ref()
        .map_or(&[], |d| d.definitions.as_slice());

    let enum_values_by_id = extract_enum_values(datatypes);

    let datatypes_by_id: HashMap<&str, &DatatypeDefinition> = datatypes
        .iter()
        .map(|d| (datatype_identifier(d), d))
        .collect();

    let spec_types: &[SpecType] = content
        .spec_types
        .as_ref()
        .map_or(&[], |s| s.types.as_slice());

    // reqif_attr_def_id → (attr long_name, entity data_type)
    let mut attr_def_info: HashMap<&str, (&str, &str)> = HashMap::new();
    for st in spec_types {
        if let SpecType::SpecObjectType(sot) = st
            && let Some(attrs) = &sot.spec_attributes
        {
            for ad in &attrs.definitions {
                let (ad_id, ad_name, dt_ref_id) = extract_attr_def_info(ad);
                if let Some(dt) = datatypes_by_id.get(dt_ref_id) {
                    let entity_type = reqif_datatype_to_entity(dt);
                    let _ = attr_def_info.insert(ad_id, (ad_name, entity_type));
                }
            }
        }
    }

    let spec_objects = content
        .spec_objects
        .as_ref()
        .map_or(&[][..], |s| s.objects.as_slice());

    let spec_relations = content
        .spec_relations
        .as_ref()
        .map_or(&[][..], |r| r.relations.as_slice());

    let now = chrono::Utc::now().fixed_offset();

    let mut total_objects = 0usize;
    let mut total_links = 0usize;
    let mut total_attr_defs = 0usize;
    let mut total_object_types = 0usize;
    let mut total_link_types = 0usize;

    // Map from ReqIF identifier → entity UUID
    let mut id_map: HashMap<String, Uuid> = HashMap::new();

    // Process each specification as a module
    let mut module_id = Uuid::now_v7();

    for spec in specifications {
        module_id = Uuid::now_v7();

        let module_name = spec
            .long_name
            .as_deref()
            .unwrap_or(&spec.identifier)
            .to_owned();

        let module_model = module::ActiveModel {
            id: Set(module_id),
            project_id: Set(project_id),
            name: Set(module_name),
            description: Set(spec.desc.clone()),
            prefix: Set("REQ".to_owned()),
            separator: Set("-".to_owned()),
            digits: Set(3),
            required_attributes: Set(json!([])),
            default_classification: Set("normative".to_owned()),
            publish_template: Set(None),
            default_lifecycle_model_id: Set(None),
            signature_config: Set(json!({})),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let _ = module_model.insert(&txn).await?;
        let _ = id_map.insert(spec.identifier.clone(), module_id);

        // Create object types + attribute definitions for SpecObjectTypes
        for st in spec_types {
            if let SpecType::SpecObjectType(sot) = st {
                let ot_id = Uuid::now_v7();
                let ot_name = sot
                    .long_name
                    .as_deref()
                    .unwrap_or(&sot.identifier)
                    .to_owned();

                let mut schema_entries = Vec::new();

                if let Some(attrs) = &sot.spec_attributes {
                    for ad in &attrs.definitions {
                        let (ad_reqif_id, ad_name, dt_ref_id) = extract_attr_def_info(ad);
                        let dt = datatypes_by_id.get(dt_ref_id);
                        let entity_type = dt.map_or("string", |d| reqif_datatype_to_entity(d));

                        let ad_id = Uuid::now_v7();

                        // Extract enum values if enumeration
                        let enum_vals = if entity_type == "enum" {
                            dt.and_then(|d| {
                                if let DatatypeDefinition::Enumeration(e) = d {
                                    e.specified_values
                                        .as_ref()
                                        .map(|sv| enum_values_to_json(&sv.values))
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        };

                        let multi_select = matches!(ad, ReqifAttrDef::Enumeration(e) if e.multi_valued == Some(true));

                        let ad_model = attribute_definition::ActiveModel {
                            id: Set(ad_id),
                            module_id: Set(Some(module_id)),
                            name: Set(ad_name.to_owned()),
                            data_type: Set(entity_type.to_owned()),
                            default_value: Set(None),
                            enum_values: Set(enum_vals),
                            multi_select: Set(multi_select),
                            depends_on: Set(None),
                            dependency_mapping: Set(None),
                            created_at: Set(now),
                        };
                        let _ = ad_model.insert(&txn).await?;

                        let _ = id_map.insert(ad_reqif_id.to_owned(), ad_id);
                        total_attr_defs += 1;

                        schema_entries.push(json!({
                            "id": ad_id.to_string(),
                            "name": ad_name,
                            "data_type": entity_type,
                        }));
                    }
                }

                let ot_model = object_type::ActiveModel {
                    id: Set(ot_id),
                    module_id: Set(module_id),
                    name: Set(ot_name),
                    description: Set(sot.desc.clone()),
                    default_classification: Set("normative".to_owned()),
                    required_attributes: Set(json!([])),
                    attribute_schema: Set(json!(schema_entries)),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                let _ = ot_model.insert(&txn).await?;

                let _ = id_map.insert(sot.identifier.clone(), ot_id);
                total_object_types += 1;
            }
        }

        // Create link types from SpecRelationTypes
        for st in spec_types {
            if let SpecType::SpecRelationType(srt) = st {
                let lt_id = Uuid::now_v7();
                let lt_name = srt
                    .long_name
                    .as_deref()
                    .unwrap_or(&srt.identifier)
                    .to_owned();

                let lt_model = link_type::ActiveModel {
                    id: Set(lt_id),
                    name: Set(lt_name),
                    description: Set(srt.desc.clone()),
                    created_at: Set(now),
                };
                let _ = lt_model.insert(&txn).await?;

                let _ = id_map.insert(srt.identifier.clone(), lt_id);
                total_link_types += 1;
            }
        }

        // Create objects from SpecObjects
        for so in spec_objects {
            let obj_id = Uuid::now_v7();
            let heading = so.long_name.clone();

            // Build attributes JSON
            let attributes = build_attributes_json(so, &attr_def_info, &enum_values_by_id);

            // Resolve object type
            let ot_uuid = id_map.get(&so.type_ref.value).copied();

            let fp = compute_content_fingerprint(heading.as_deref(), None, attributes.as_ref());

            let obj_model = object::ActiveModel {
                id: Set(obj_id),
                module_id: Set(module_id),
                parent_id: Set(None),
                position: Set(0),
                level: Set("0".to_owned()),
                heading: Set(heading),
                body: Set(None),
                attributes: Set(attributes),
                current_version: Set(1),
                classification: Set("normative".to_owned()),
                content_fingerprint: Set(fp),
                reviewed_fingerprint: Set(None),
                reviewed_at: Set(None),
                reviewed_by: Set(None),
                references_: Set(json!([])),
                object_type_id: Set(ot_uuid),
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
            let _ = obj_model.insert(&txn).await?;

            let _ = id_map.insert(so.identifier.clone(), obj_id);
            total_objects += 1;
        }

        // Walk SpecHierarchy tree to set parent_id and position
        if let Some(children) = &spec.children {
            walk_hierarchy(&txn, children, None, &id_map, now).await?;
        }

        // Create links from SpecRelations
        for sr in spec_relations {
            let source_uuid = id_map.get(sr.source.spec_object_ref.as_str()).copied();
            let target_uuid = id_map.get(sr.target.spec_object_ref.as_str()).copied();
            let lt_uuid = id_map.get(sr.type_ref.value.as_str()).copied();

            if let (Some(src), Some(tgt), Some(lt)) = (source_uuid, target_uuid, lt_uuid) {
                let link_id = Uuid::now_v7();

                let link_model = link::ActiveModel {
                    id: Set(link_id),
                    source_object_id: Set(src),
                    target_object_id: Set(tgt),
                    link_type_id: Set(lt),
                    attributes: Set(None),
                    suspect: Set(false),
                    source_fingerprint: Set(String::new()),
                    target_fingerprint: Set(String::new()),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                let _ = link_model.insert(&txn).await?;
                total_links += 1;
            }
        }
    }

    txn.commit().await?;

    Ok(ImportResult {
        module_id,
        objects_created: total_objects,
        links_created: total_links,
        attribute_definitions_created: total_attr_defs,
        object_types_created: total_object_types,
        link_types_created: total_link_types,
    })
}

/// Build the attributes JSON object for a `SpecObject` from its `AttributeValues`.
fn build_attributes_json(
    so: &req1_reqif::SpecObject,
    attr_def_info: &HashMap<&str, (&str, &str)>,
    enum_values_by_id: &HashMap<String, String>,
) -> Option<serde_json::Value> {
    let values = so.values.as_ref()?;
    if values.values.is_empty() {
        return None;
    }

    let mut attrs = serde_json::Map::new();
    for av in &values.values {
        let def_id = attr_value_def_ref(av);
        let attr_name = attr_def_info.get(def_id).map_or(def_id, |(name, _)| *name);

        let json_val = reqif_attr_value_to_json(av, enum_values_by_id);
        let _ = attrs.insert(attr_name.to_owned(), json_val);
    }

    Some(serde_json::Value::Object(attrs))
}

/// Recursively walk a `SpecHierarchy` tree, setting `parent_id` and `position` on objects.
async fn walk_hierarchy(
    db: &impl ConnectionTrait,
    children: &SpecHierarchyChildren,
    parent_uuid: Option<Uuid>,
    id_map: &HashMap<String, Uuid>,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<(), CoreError> {
    for (pos, sh) in children.hierarchies.iter().enumerate() {
        let obj_reqif_id = &sh.object.spec_object_ref;
        if let Some(&obj_uuid) = id_map.get(obj_reqif_id.as_str()) {
            let update = object::ActiveModel {
                id: Set(obj_uuid),
                parent_id: Set(parent_uuid),
                position: Set(i32::try_from(pos).unwrap_or(0)),
                updated_at: Set(now),
                ..Default::default()
            };
            let _ = update.update(db).await?;

            if let Some(sub_children) = &sh.children {
                Box::pin(walk_hierarchy(
                    db,
                    sub_children,
                    Some(obj_uuid),
                    id_map,
                    now,
                ))
                .await?;
            }
        }
    }
    Ok(())
}

/// Extract (identifier, `long_name`, `datatype_ref_id`) from a `ReqIF` `AttributeDefinition`.
fn extract_attr_def_info(ad: &ReqifAttrDef) -> (&str, &str, &str) {
    match ad {
        ReqifAttrDef::Boolean(d) => (
            d.identifier.as_str(),
            d.long_name.as_deref().unwrap_or(&d.identifier),
            attr_def_type_ref_id(&d.datatype_ref),
        ),
        ReqifAttrDef::Date(d) => (
            d.identifier.as_str(),
            d.long_name.as_deref().unwrap_or(&d.identifier),
            attr_def_type_ref_id(&d.datatype_ref),
        ),
        ReqifAttrDef::Enumeration(d) => (
            d.identifier.as_str(),
            d.long_name.as_deref().unwrap_or(&d.identifier),
            attr_def_type_ref_id(&d.datatype_ref),
        ),
        ReqifAttrDef::Integer(d) => (
            d.identifier.as_str(),
            d.long_name.as_deref().unwrap_or(&d.identifier),
            attr_def_type_ref_id(&d.datatype_ref),
        ),
        ReqifAttrDef::Real(d) => (
            d.identifier.as_str(),
            d.long_name.as_deref().unwrap_or(&d.identifier),
            attr_def_type_ref_id(&d.datatype_ref),
        ),
        ReqifAttrDef::Str(d) => (
            d.identifier.as_str(),
            d.long_name.as_deref().unwrap_or(&d.identifier),
            attr_def_type_ref_id(&d.datatype_ref),
        ),
        ReqifAttrDef::Xhtml(d) => (
            d.identifier.as_str(),
            d.long_name.as_deref().unwrap_or(&d.identifier),
            attr_def_type_ref_id(&d.datatype_ref),
        ),
    }
}

fn attr_def_type_ref_id(r: &req1_reqif::AttrDefTypeRef) -> &str {
    use req1_reqif::AttrDefTypeRefInner;
    match &r.inner {
        AttrDefTypeRefInner::Boolean(id)
        | AttrDefTypeRefInner::Date(id)
        | AttrDefTypeRefInner::Enumeration(id)
        | AttrDefTypeRefInner::Integer(id)
        | AttrDefTypeRefInner::Real(id)
        | AttrDefTypeRefInner::Str(id)
        | AttrDefTypeRefInner::Xhtml(id) => id,
    }
}
