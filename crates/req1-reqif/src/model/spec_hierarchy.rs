use serde::{Deserialize, Serialize};

use super::common::identifiable_struct;
use super::spec_objects::AttributeValues;

// ---------------------------------------------------------------------------
// Specifications container
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "SPECIFICATIONS")]
pub struct Specifications {
    #[serde(rename = "SPECIFICATION", default)]
    pub specifications: Vec<Specification>,
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename = "SPECIFICATION")]
    pub struct Specification {
        #[serde(rename = "TYPE")]
        type_ref: SpecificationTypeRef,

        #[serde(rename = "VALUES", skip_serializing_if = "Option::is_none", default)]
        values: Option<AttributeValues>,

        #[serde(rename = "CHILDREN", skip_serializing_if = "Option::is_none", default)]
        children: Option<SpecHierarchyChildren>,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecificationTypeRef {
    #[serde(rename = "SPECIFICATION-TYPE-REF")]
    pub value: String,
}

// ---------------------------------------------------------------------------
// SpecHierarchy (recursive tree)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "CHILDREN")]
pub struct SpecHierarchyChildren {
    #[serde(rename = "SPEC-HIERARCHY", default)]
    pub hierarchies: Vec<SpecHierarchy>,
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename = "SPEC-HIERARCHY")]
    pub struct SpecHierarchy {
        #[serde(rename = "@IS-TABLE-INTERNAL", skip_serializing_if = "Option::is_none", default)]
        is_table_internal: Option<bool>,

        #[serde(rename = "OBJECT")]
        object: SpecHierarchyObjectRef,

        #[serde(rename = "CHILDREN", skip_serializing_if = "Option::is_none", default)]
        children: Option<SpecHierarchyChildren>,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecHierarchyObjectRef {
    #[serde(rename = "SPEC-OBJECT-REF")]
    pub spec_object_ref: String,
}
