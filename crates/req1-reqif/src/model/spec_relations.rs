use serde::{Deserialize, Serialize};

use super::common::identifiable_struct;
use super::spec_objects::AttributeValues;

// ---------------------------------------------------------------------------
// SpecRelations container
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "SPEC-RELATIONS")]
pub struct SpecRelations {
    #[serde(rename = "SPEC-RELATION", default)]
    pub relations: Vec<SpecRelation>,
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename = "SPEC-RELATION")]
    pub struct SpecRelation {
        #[serde(rename = "TYPE")]
        type_ref: SpecRelationTypeRef,

        #[serde(rename = "SOURCE")]
        source: SpecRelationEndpoint,

        #[serde(rename = "TARGET")]
        target: SpecRelationEndpoint,

        #[serde(rename = "VALUES", skip_serializing_if = "Option::is_none", default)]
        values: Option<AttributeValues>,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecRelationTypeRef {
    #[serde(rename = "SPEC-RELATION-TYPE-REF")]
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecRelationEndpoint {
    #[serde(rename = "SPEC-OBJECT-REF")]
    pub spec_object_ref: String,
}

// ---------------------------------------------------------------------------
// SpecRelationGroups container
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "SPEC-RELATION-GROUPS")]
pub struct SpecRelationGroups {
    #[serde(rename = "RELATION-GROUP", default)]
    pub groups: Vec<RelationGroup>,
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename = "RELATION-GROUP")]
    pub struct RelationGroup {
        #[serde(rename = "TYPE")]
        type_ref: RelationGroupTypeRef,

        #[serde(rename = "SOURCE-SPECIFICATION")]
        source_specification: SpecificationRef,

        #[serde(rename = "TARGET-SPECIFICATION")]
        target_specification: SpecificationRef,

        #[serde(rename = "SPEC-RELATIONS", skip_serializing_if = "Option::is_none", default)]
        spec_relations: Option<RelationGroupSpecRelations>,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationGroupTypeRef {
    #[serde(rename = "RELATION-GROUP-TYPE-REF")]
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecificationRef {
    #[serde(rename = "SPECIFICATION-REF")]
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationGroupSpecRelations {
    #[serde(rename = "SPEC-RELATION-REF", default)]
    pub refs: Vec<String>,
}
