use serde::{Deserialize, Serialize};

use super::common::identifiable_struct;

// ---------------------------------------------------------------------------
// Container
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "SPEC-TYPES")]
pub struct SpecTypes {
    #[serde(rename = "$value", default)]
    pub types: Vec<SpecType>,
}

// ---------------------------------------------------------------------------
// Enum dispatch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpecType {
    #[serde(rename = "SPEC-OBJECT-TYPE")]
    SpecObjectType(SpecObjectType),
    #[serde(rename = "SPEC-RELATION-TYPE")]
    SpecRelationType(SpecRelationType),
    #[serde(rename = "SPECIFICATION-TYPE")]
    SpecificationType(SpecificationType),
    #[serde(rename = "RELATION-GROUP-TYPE")]
    RelationGroupType(RelationGroupType),
}

// ---------------------------------------------------------------------------
// Type structs
// ---------------------------------------------------------------------------

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SpecObjectType {
        #[serde(rename = "SPEC-ATTRIBUTES", skip_serializing_if = "Option::is_none", default)]
        spec_attributes: Option<SpecAttributes>,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SpecRelationType {
        #[serde(rename = "SPEC-ATTRIBUTES", skip_serializing_if = "Option::is_none", default)]
        spec_attributes: Option<SpecAttributes>,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SpecificationType {
        #[serde(rename = "SPEC-ATTRIBUTES", skip_serializing_if = "Option::is_none", default)]
        spec_attributes: Option<SpecAttributes>,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct RelationGroupType {
        #[serde(rename = "SPEC-ATTRIBUTES", skip_serializing_if = "Option::is_none", default)]
        spec_attributes: Option<SpecAttributes>,
    }
}

// ---------------------------------------------------------------------------
// Attribute definitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "SPEC-ATTRIBUTES")]
pub struct SpecAttributes {
    #[serde(rename = "$value", default)]
    pub definitions: Vec<AttributeDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeDefinition {
    #[serde(rename = "ATTRIBUTE-DEFINITION-BOOLEAN")]
    Boolean(AttributeDefinitionBoolean),
    #[serde(rename = "ATTRIBUTE-DEFINITION-DATE")]
    Date(AttributeDefinitionDate),
    #[serde(rename = "ATTRIBUTE-DEFINITION-ENUMERATION")]
    Enumeration(AttributeDefinitionEnumeration),
    #[serde(rename = "ATTRIBUTE-DEFINITION-INTEGER")]
    Integer(AttributeDefinitionInteger),
    #[serde(rename = "ATTRIBUTE-DEFINITION-REAL")]
    Real(AttributeDefinitionReal),
    #[serde(rename = "ATTRIBUTE-DEFINITION-STRING")]
    Str(AttributeDefinitionString),
    #[serde(rename = "ATTRIBUTE-DEFINITION-XHTML")]
    Xhtml(AttributeDefinitionXhtml),
}

// ---------------------------------------------------------------------------
// Attribute definition structs
// ---------------------------------------------------------------------------

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AttributeDefinitionBoolean {
        #[serde(rename = "@IS-EDITABLE", skip_serializing_if = "Option::is_none", default)]
        is_editable: Option<bool>,

        #[serde(rename = "TYPE")]
        datatype_ref: AttrDefTypeRef,

        #[serde(rename = "DEFAULT-VALUE", skip_serializing_if = "Option::is_none", default)]
        default_value: Option<DefaultValueBoolean>,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultValueBoolean {
    #[serde(rename = "ATTRIBUTE-VALUE-BOOLEAN")]
    pub value: DefaultValueBooleanInner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultValueBooleanInner {
    #[serde(rename = "@THE-VALUE")]
    pub the_value: bool,
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AttributeDefinitionDate {
        #[serde(rename = "@IS-EDITABLE", skip_serializing_if = "Option::is_none", default)]
        is_editable: Option<bool>,

        #[serde(rename = "TYPE")]
        datatype_ref: AttrDefTypeRef,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AttributeDefinitionEnumeration {
        #[serde(rename = "@IS-EDITABLE", skip_serializing_if = "Option::is_none", default)]
        is_editable: Option<bool>,

        #[serde(rename = "@MULTI-VALUED", skip_serializing_if = "Option::is_none", default)]
        multi_valued: Option<bool>,

        #[serde(rename = "TYPE")]
        datatype_ref: AttrDefTypeRef,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AttributeDefinitionInteger {
        #[serde(rename = "@IS-EDITABLE", skip_serializing_if = "Option::is_none", default)]
        is_editable: Option<bool>,

        #[serde(rename = "TYPE")]
        datatype_ref: AttrDefTypeRef,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AttributeDefinitionReal {
        #[serde(rename = "@IS-EDITABLE", skip_serializing_if = "Option::is_none", default)]
        is_editable: Option<bool>,

        #[serde(rename = "TYPE")]
        datatype_ref: AttrDefTypeRef,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AttributeDefinitionString {
        #[serde(rename = "@IS-EDITABLE", skip_serializing_if = "Option::is_none", default)]
        is_editable: Option<bool>,

        #[serde(rename = "TYPE")]
        datatype_ref: AttrDefTypeRef,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AttributeDefinitionXhtml {
        #[serde(rename = "@IS-EDITABLE", skip_serializing_if = "Option::is_none", default)]
        is_editable: Option<bool>,

        #[serde(rename = "TYPE")]
        datatype_ref: AttrDefTypeRef,
    }
}

// ---------------------------------------------------------------------------
// Type reference wrappers
// ---------------------------------------------------------------------------

/// The `<TYPE>` wrapper element containing a datatype definition reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttrDefTypeRef {
    #[serde(rename = "$value")]
    pub inner: AttrDefTypeRefInner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttrDefTypeRefInner {
    #[serde(rename = "DATATYPE-DEFINITION-BOOLEAN-REF")]
    Boolean(String),
    #[serde(rename = "DATATYPE-DEFINITION-DATE-REF")]
    Date(String),
    #[serde(rename = "DATATYPE-DEFINITION-ENUMERATION-REF")]
    Enumeration(String),
    #[serde(rename = "DATATYPE-DEFINITION-INTEGER-REF")]
    Integer(String),
    #[serde(rename = "DATATYPE-DEFINITION-REAL-REF")]
    Real(String),
    #[serde(rename = "DATATYPE-DEFINITION-STRING-REF")]
    Str(String),
    #[serde(rename = "DATATYPE-DEFINITION-XHTML-REF")]
    Xhtml(String),
}
