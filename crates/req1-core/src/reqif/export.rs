use std::collections::HashMap;

use sea_orm::{ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use entity::{attribute_definition, link, link_type, module, object, object_type};
use req1_reqif::{
    AttrDefTypeRef, AttrDefTypeRefInner, AttributeDefinition as ReqifAttrDef,
    AttributeDefinitionBoolean, AttributeDefinitionDate, AttributeDefinitionEnumeration,
    AttributeDefinitionInteger, AttributeDefinitionReal, AttributeDefinitionString,
    AttributeDefinitionXhtml, AttributeValues, Datatypes, EmbeddedValue, EnumValue,
    EnumValueProperties, ReqIfBuilder, SpecAttributes, SpecHierarchy, SpecHierarchyChildren,
    SpecHierarchyObjectRef, SpecObjectType, SpecObjectTypeRef, SpecObjects, SpecRelation,
    SpecRelationEndpoint, SpecRelationType, SpecRelationTypeRef, SpecRelations, SpecType,
    SpecTypes, Specification, SpecificationTypeRef, Specifications, SpecificationType,
    SpecifiedValues,
};

use crate::error::CoreError;

use super::type_map::{entity_datatype_to_reqif, json_to_reqif_attr_value};
use super::ExportResult;

/// Export a module and its contents to a `ReqIF` document.
#[allow(clippy::too_many_lines)]
pub async fn export_reqif(
    db: &impl ConnectionTrait,
    module_id: Uuid,
) -> Result<ExportResult, CoreError> {
    let module_entity = module::Entity::find_by_id(module_id)
        .one(db)
        .await?
        .ok_or_else(|| CoreError::NotFound(format!("module {module_id}")))?;

    let objects: Vec<object::Model> = object::Entity::find()
        .filter(object::Column::ModuleId.eq(module_id))
        .filter(object::Column::DeletedAt.is_null())
        .order_by_asc(object::Column::Position)
        .all(db)
        .await?;

    let attr_defs: Vec<attribute_definition::Model> = attribute_definition::Entity::find()
        .filter(attribute_definition::Column::ModuleId.eq(Some(module_id)))
        .all(db)
        .await?;

    let obj_types: Vec<object_type::Model> = object_type::Entity::find()
        .filter(object_type::Column::ModuleId.eq(module_id))
        .all(db)
        .await?;

    let object_ids: Vec<Uuid> = objects.iter().map(|o| o.id).collect();

    let links: Vec<link::Model> = if object_ids.is_empty() {
        Vec::new()
    } else {
        link::Entity::find()
            .filter(
                Condition::any()
                    .add(link::Column::SourceObjectId.is_in(object_ids.clone()))
                    .add(link::Column::TargetObjectId.is_in(object_ids)),
            )
            .all(db)
            .await?
    };

    let link_type_ids: Vec<Uuid> = links.iter().map(|l| l.link_type_id).collect();
    let link_types: Vec<link_type::Model> = if link_type_ids.is_empty() {
        Vec::new()
    } else {
        link_type::Entity::find()
            .filter(link_type::Column::Id.is_in(link_type_ids))
            .all(db)
            .await?
    };

    let (reqif_datatypes, datatype_map) = build_datatypes(&attr_defs);

    let mut attr_def_reqif_ids: HashMap<Uuid, String> = HashMap::new();
    let mut attr_def_data_types: HashMap<Uuid, String> = HashMap::new();
    for ad in &attr_defs {
        let reqif_id = format!("req1-{}", ad.id);
        let _ = attr_def_reqif_ids.insert(ad.id, reqif_id);
        let _ = attr_def_data_types.insert(ad.id, ad.data_type.clone());
    }

    let enum_name_to_id = build_enum_name_to_id_map(&reqif_datatypes);

    let spec_type_list = build_spec_types(
        &obj_types,
        &link_types,
        &attr_defs,
        &datatype_map,
        &module_entity,
        module_id,
    );

    let (reqif_objects, object_reqif_ids) = build_spec_objects(
        &objects,
        &obj_types,
        module_id,
        &attr_defs,
        &attr_def_reqif_ids,
        &attr_def_data_types,
        &enum_name_to_id,
    );

    let hierarchy = build_hierarchy_tree(&objects, &object_reqif_ids);
    let reqif_relations = build_spec_relations(&links, &object_reqif_ids);

    let specification = Specification {
        identifier: format!("req1-{module_id}"),
        long_name: Some(module_entity.name.clone()),
        last_change: None,
        desc: module_entity.description.clone(),
        type_ref: SpecificationTypeRef {
            value: format!("req1-spectype-{module_id}"),
        },
        values: None,
        children: if hierarchy.is_empty() {
            None
        } else {
            Some(SpecHierarchyChildren {
                hierarchies: hierarchy,
            })
        },
    };

    let objects_exported = reqif_objects.len();
    let links_exported = reqif_relations.len();

    let mut builder =
        ReqIfBuilder::new(format!("req1-export-{module_id}"), module_entity.name.clone());

    if !reqif_datatypes.is_empty() {
        builder = builder.datatypes(Datatypes {
            definitions: reqif_datatypes,
        });
    }
    if !spec_type_list.is_empty() {
        builder = builder.spec_types(SpecTypes {
            types: spec_type_list,
        });
    }
    if !reqif_objects.is_empty() {
        builder = builder.spec_objects(SpecObjects {
            objects: reqif_objects,
        });
    }
    if !reqif_relations.is_empty() {
        builder = builder.spec_relations(SpecRelations {
            relations: reqif_relations,
        });
    }
    builder = builder.specifications(Specifications {
        specifications: vec![specification],
    });

    Ok(ExportResult {
        document: builder.build(),
        objects_exported,
        links_exported,
    })
}

fn build_datatypes(
    attr_defs: &[attribute_definition::Model],
) -> (Vec<req1_reqif::DatatypeDefinition>, HashMap<String, String>) {
    let mut datatype_map: HashMap<String, String> = HashMap::new();
    let mut reqif_datatypes = Vec::new();

    for ad in attr_defs {
        if datatype_map.contains_key(&ad.data_type) {
            continue;
        }
        let dt_id = format!("req1-dt-{}", ad.data_type);
        let mut dt = entity_datatype_to_reqif(&ad.data_type, &dt_id);

        if ad.data_type == "enum"
            && let req1_reqif::DatatypeDefinition::Enumeration(ref mut e) = dt
            && let Some(vals) = &ad.enum_values
            && let Some(arr) = vals.as_array()
        {
            let enum_vals: Vec<EnumValue> = arr
                .iter()
                .enumerate()
                .filter_map(|(idx, v)| {
                    v.as_str().map(|s| EnumValue {
                        identifier: format!("req1-ev-{}-{idx}", ad.id),
                        long_name: Some(s.to_owned()),
                        last_change: None,
                        desc: None,
                        properties: Some(EnumValueProperties {
                            embedded_value: EmbeddedValue {
                                key: i64::try_from(idx).unwrap_or(0),
                                other_content: None,
                            },
                        }),
                    })
                })
                .collect();
            e.specified_values = Some(SpecifiedValues { values: enum_vals });
        }

        let _ = datatype_map.insert(ad.data_type.clone(), dt_id);
        reqif_datatypes.push(dt);
    }

    (reqif_datatypes, datatype_map)
}

fn build_spec_types(
    obj_types: &[object_type::Model],
    link_types: &[link_type::Model],
    attr_defs: &[attribute_definition::Model],
    datatype_map: &HashMap<String, String>,
    module_entity: &module::Model,
    module_id: Uuid,
) -> Vec<SpecType> {
    let mut list: Vec<SpecType> = Vec::new();

    for ot in obj_types {
        let ot_reqif_id = format!("req1-{}", ot.id);
        let mut spec_attrs = Vec::new();

        for ad in attr_defs {
            let ad_reqif_id = format!("req1-{}", ad.id);
            let dt_ref_id = datatype_map
                .get(&ad.data_type)
                .cloned()
                .unwrap_or_default();

            spec_attrs.push(build_reqif_attr_def(
                &ad.name,
                &ad_reqif_id,
                &ad.data_type,
                &dt_ref_id,
                ad.multi_select,
            ));
        }

        list.push(SpecType::SpecObjectType(SpecObjectType {
            identifier: ot_reqif_id,
            long_name: Some(ot.name.clone()),
            last_change: None,
            desc: ot.description.clone(),
            spec_attributes: if spec_attrs.is_empty() {
                None
            } else {
                Some(SpecAttributes {
                    definitions: spec_attrs,
                })
            },
        }));
    }

    for lt in link_types {
        list.push(SpecType::SpecRelationType(SpecRelationType {
            identifier: format!("req1-{}", lt.id),
            long_name: Some(lt.name.clone()),
            last_change: None,
            desc: lt.description.clone(),
            spec_attributes: None,
        }));
    }

    list.push(SpecType::SpecificationType(SpecificationType {
        identifier: format!("req1-spectype-{module_id}"),
        long_name: Some(module_entity.name.clone()),
        last_change: None,
        desc: module_entity.description.clone(),
        spec_attributes: None,
    }));

    list
}

#[allow(clippy::too_many_arguments)]
fn build_spec_objects(
    objects: &[object::Model],
    obj_types: &[object_type::Model],
    module_id: Uuid,
    attr_defs: &[attribute_definition::Model],
    attr_def_reqif_ids: &HashMap<Uuid, String>,
    attr_def_data_types: &HashMap<Uuid, String>,
    enum_name_to_id: &HashMap<String, String>,
) -> (Vec<req1_reqif::SpecObject>, HashMap<Uuid, String>) {
    let mut reqif_objects = Vec::new();
    let mut object_reqif_ids: HashMap<Uuid, String> = HashMap::new();

    for obj in objects {
        let obj_reqif_id = format!("req1-{}", obj.id);
        let _ = object_reqif_ids.insert(obj.id, obj_reqif_id.clone());

        let type_ref_id = obj.object_type_id.map_or_else(
            || {
                obj_types
                    .first()
                    .map_or_else(
                        || format!("req1-default-type-{module_id}"),
                        |ot| format!("req1-{}", ot.id),
                    )
            },
            |ot_id| format!("req1-{ot_id}"),
        );

        let attr_values = build_attr_values(
            obj.attributes.as_ref(),
            attr_defs,
            attr_def_reqif_ids,
            attr_def_data_types,
            enum_name_to_id,
        );

        reqif_objects.push(req1_reqif::SpecObject {
            identifier: obj_reqif_id,
            long_name: obj.heading.clone(),
            last_change: None,
            desc: None,
            type_ref: SpecObjectTypeRef {
                value: type_ref_id,
            },
            values: attr_values,
        });
    }

    (reqif_objects, object_reqif_ids)
}

fn build_spec_relations(
    links: &[link::Model],
    object_reqif_ids: &HashMap<Uuid, String>,
) -> Vec<SpecRelation> {
    links
        .iter()
        .map(|lnk| {
            let src_id = object_reqif_ids
                .get(&lnk.source_object_id)
                .cloned()
                .unwrap_or_else(|| format!("req1-{}", lnk.source_object_id));
            let tgt_id = object_reqif_ids
                .get(&lnk.target_object_id)
                .cloned()
                .unwrap_or_else(|| format!("req1-{}", lnk.target_object_id));

            SpecRelation {
                identifier: format!("req1-{}", lnk.id),
                long_name: None,
                last_change: None,
                desc: None,
                type_ref: SpecRelationTypeRef {
                    value: format!("req1-{}", lnk.link_type_id),
                },
                source: SpecRelationEndpoint {
                    spec_object_ref: src_id,
                },
                target: SpecRelationEndpoint {
                    spec_object_ref: tgt_id,
                },
                values: None,
            }
        })
        .collect()
}

fn build_attr_values(
    attributes: Option<&serde_json::Value>,
    attr_defs: &[attribute_definition::Model],
    attr_def_reqif_ids: &HashMap<Uuid, String>,
    attr_def_data_types: &HashMap<Uuid, String>,
    enum_name_to_id: &HashMap<String, String>,
) -> Option<AttributeValues> {
    let attrs = attributes?.as_object()?;
    if attrs.is_empty() {
        return None;
    }

    let mut values = Vec::new();

    for ad in attr_defs {
        if let Some(json_val) = attrs.get(&ad.name) {
            let reqif_id = attr_def_reqif_ids.get(&ad.id)?;
            let data_type = attr_def_data_types.get(&ad.id)?;

            if let Some(av) =
                json_to_reqif_attr_value(json_val, reqif_id, data_type, enum_name_to_id)
            {
                values.push(av);
            }
        }
    }

    if values.is_empty() {
        None
    } else {
        Some(AttributeValues { values })
    }
}

/// Build the `SpecHierarchy` tree from the flat list of objects with `parent_id`.
fn build_hierarchy_tree(
    objects: &[object::Model],
    object_reqif_ids: &HashMap<Uuid, String>,
) -> Vec<SpecHierarchy> {
    let mut children_map: HashMap<Option<Uuid>, Vec<&object::Model>> = HashMap::new();
    for obj in objects {
        children_map.entry(obj.parent_id).or_default().push(obj);
    }
    for group in children_map.values_mut() {
        group.sort_by_key(|o| o.position);
    }
    build_children(None, &children_map, object_reqif_ids)
}

fn build_children(
    parent: Option<Uuid>,
    children_map: &HashMap<Option<Uuid>, Vec<&object::Model>>,
    object_reqif_ids: &HashMap<Uuid, String>,
) -> Vec<SpecHierarchy> {
    let Some(children) = children_map.get(&parent) else {
        return Vec::new();
    };

    children
        .iter()
        .map(|obj| {
            let obj_reqif_id = object_reqif_ids
                .get(&obj.id)
                .cloned()
                .unwrap_or_else(|| format!("req1-{}", obj.id));

            let sub = build_children(Some(obj.id), children_map, object_reqif_ids);

            SpecHierarchy {
                identifier: format!("req1-sh-{}", obj.id),
                long_name: None,
                last_change: None,
                desc: None,
                is_table_internal: None,
                object: SpecHierarchyObjectRef {
                    spec_object_ref: obj_reqif_id,
                },
                children: if sub.is_empty() {
                    None
                } else {
                    Some(SpecHierarchyChildren { hierarchies: sub })
                },
            }
        })
        .collect()
}

/// Build a `ReqIF` `AttributeDefinition` from entity data.
fn build_reqif_attr_def(
    name: &str,
    reqif_id: &str,
    data_type: &str,
    dt_ref_id: &str,
    multi_select: bool,
) -> ReqifAttrDef {
    let id = reqif_id.to_owned();
    let long = Some(name.to_owned());
    let dt = dt_ref_id.to_owned();

    match data_type {
        "boolean" => ReqifAttrDef::Boolean(AttributeDefinitionBoolean {
            identifier: id,
            long_name: long,
            last_change: None,
            desc: None,
            is_editable: Some(true),
            datatype_ref: AttrDefTypeRef {
                inner: AttrDefTypeRefInner::Boolean(dt),
            },
            default_value: None,
        }),
        "date" => ReqifAttrDef::Date(AttributeDefinitionDate {
            identifier: id,
            long_name: long,
            last_change: None,
            desc: None,
            is_editable: Some(true),
            datatype_ref: AttrDefTypeRef {
                inner: AttrDefTypeRefInner::Date(dt),
            },
        }),
        "enum" => ReqifAttrDef::Enumeration(AttributeDefinitionEnumeration {
            identifier: id,
            long_name: long,
            last_change: None,
            desc: None,
            is_editable: Some(true),
            multi_valued: if multi_select { Some(true) } else { None },
            datatype_ref: AttrDefTypeRef {
                inner: AttrDefTypeRefInner::Enumeration(dt),
            },
        }),
        "integer" => ReqifAttrDef::Integer(AttributeDefinitionInteger {
            identifier: id,
            long_name: long,
            last_change: None,
            desc: None,
            is_editable: Some(true),
            datatype_ref: AttrDefTypeRef {
                inner: AttrDefTypeRefInner::Integer(dt),
            },
        }),
        "float" => ReqifAttrDef::Real(AttributeDefinitionReal {
            identifier: id,
            long_name: long,
            last_change: None,
            desc: None,
            is_editable: Some(true),
            datatype_ref: AttrDefTypeRef {
                inner: AttrDefTypeRefInner::Real(dt),
            },
        }),
        "string" => ReqifAttrDef::Str(AttributeDefinitionString {
            identifier: id,
            long_name: long,
            last_change: None,
            desc: None,
            is_editable: Some(true),
            datatype_ref: AttrDefTypeRef {
                inner: AttrDefTypeRefInner::Str(dt),
            },
        }),
        _ => ReqifAttrDef::Xhtml(AttributeDefinitionXhtml {
            identifier: id,
            long_name: long,
            last_change: None,
            desc: None,
            is_editable: Some(true),
            datatype_ref: AttrDefTypeRef {
                inner: AttrDefTypeRefInner::Xhtml(dt),
            },
        }),
    }
}

/// Build a map of enum value `long_name` to `ReqIF` enum value identifier for export.
fn build_enum_name_to_id_map(
    datatypes: &[req1_reqif::DatatypeDefinition],
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for dt in datatypes {
        if let req1_reqif::DatatypeDefinition::Enumeration(e) = dt
            && let Some(sv) = &e.specified_values
        {
            for ev in &sv.values {
                let name = ev.long_name.clone().unwrap_or_else(|| ev.identifier.clone());
                let _ = map.insert(name, ev.identifier.clone());
            }
        }
    }
    map
}
