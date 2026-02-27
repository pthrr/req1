pub mod common;
pub mod datatypes;
pub mod header;
pub mod spec_hierarchy;
pub mod spec_objects;
pub mod spec_relations;
pub mod spec_types;

use serde::{Deserialize, Serialize};

pub use self::common::ReqifDateTime;
pub use self::datatypes::*;
pub use self::header::ReqIfHeader;
pub use self::spec_hierarchy::*;
pub use self::spec_objects::*;
pub use self::spec_relations::*;
pub use self::spec_types::*;

// ---------------------------------------------------------------------------
// Root document
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "REQ-IF")]
pub struct ReqIf {
    #[serde(rename = "@xmlns", skip_serializing_if = "Option::is_none", default)]
    pub xmlns: Option<String>,

    #[serde(rename = "THE-HEADER")]
    pub the_header: TheHeader,

    #[serde(rename = "CORE-CONTENT")]
    pub core_content: CoreContent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "THE-HEADER")]
pub struct TheHeader {
    #[serde(rename = "REQ-IF-HEADER")]
    pub req_if_header: ReqIfHeader,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "CORE-CONTENT")]
pub struct CoreContent {
    #[serde(rename = "REQ-IF-CONTENT")]
    pub req_if_content: ReqIfContent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "REQ-IF-CONTENT")]
pub struct ReqIfContent {
    #[serde(rename = "DATATYPES", skip_serializing_if = "Option::is_none", default)]
    pub datatypes: Option<Datatypes>,

    #[serde(rename = "SPEC-TYPES", skip_serializing_if = "Option::is_none", default)]
    pub spec_types: Option<SpecTypes>,

    #[serde(rename = "SPEC-OBJECTS", skip_serializing_if = "Option::is_none", default)]
    pub spec_objects: Option<SpecObjects>,

    #[serde(rename = "SPEC-RELATIONS", skip_serializing_if = "Option::is_none", default)]
    pub spec_relations: Option<SpecRelations>,

    #[serde(rename = "SPECIFICATIONS", skip_serializing_if = "Option::is_none", default)]
    pub specifications: Option<Specifications>,

    #[serde(rename = "SPEC-RELATION-GROUPS", skip_serializing_if = "Option::is_none", default)]
    pub spec_relation_groups: Option<SpecRelationGroups>,
}
