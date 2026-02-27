use serde::{Deserialize, Serialize};

use super::common::identifiable_struct;

// ---------------------------------------------------------------------------
// Container
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "SPEC-OBJECTS")]
pub struct SpecObjects {
    #[serde(rename = "SPEC-OBJECT", default)]
    pub objects: Vec<SpecObject>,
}

// ---------------------------------------------------------------------------
// SpecObject
// ---------------------------------------------------------------------------

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename = "SPEC-OBJECT")]
    pub struct SpecObject {
        #[serde(rename = "TYPE")]
        type_ref: SpecObjectTypeRef,

        #[serde(rename = "VALUES", skip_serializing_if = "Option::is_none", default)]
        values: Option<AttributeValues>,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecObjectTypeRef {
    #[serde(rename = "SPEC-OBJECT-TYPE-REF")]
    pub value: String,
}

// ---------------------------------------------------------------------------
// Attribute values
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "VALUES")]
pub struct AttributeValues {
    #[serde(rename = "$value", default)]
    pub values: Vec<AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    #[serde(rename = "ATTRIBUTE-VALUE-BOOLEAN")]
    Boolean(AttributeValueBoolean),
    #[serde(rename = "ATTRIBUTE-VALUE-DATE")]
    Date(AttributeValueDate),
    #[serde(rename = "ATTRIBUTE-VALUE-ENUMERATION")]
    Enumeration(AttributeValueEnumeration),
    #[serde(rename = "ATTRIBUTE-VALUE-INTEGER")]
    Integer(AttributeValueInteger),
    #[serde(rename = "ATTRIBUTE-VALUE-REAL")]
    Real(AttributeValueReal),
    #[serde(rename = "ATTRIBUTE-VALUE-STRING")]
    Str(AttributeValueString),
    #[serde(rename = "ATTRIBUTE-VALUE-XHTML")]
    Xhtml(AttributeValueXhtml),
}

// ---------------------------------------------------------------------------
// Value variant structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeValueBoolean {
    #[serde(rename = "@THE-VALUE")]
    pub the_value: bool,

    #[serde(rename = "DEFINITION")]
    pub definition: AttributeValueDefinitionRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeValueDate {
    #[serde(rename = "@THE-VALUE")]
    pub the_value: String,

    #[serde(rename = "DEFINITION")]
    pub definition: AttributeValueDefinitionRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeValueEnumeration {
    #[serde(rename = "VALUES", skip_serializing_if = "Option::is_none", default)]
    pub values: Option<EnumValueRefs>,

    #[serde(rename = "DEFINITION")]
    pub definition: AttributeValueDefinitionRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumValueRefs {
    #[serde(rename = "ENUM-VALUE-REF", default)]
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeValueInteger {
    #[serde(rename = "@THE-VALUE")]
    pub the_value: i64,

    #[serde(rename = "DEFINITION")]
    pub definition: AttributeValueDefinitionRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeValueReal {
    #[serde(rename = "@THE-VALUE")]
    pub the_value: f64,

    #[serde(rename = "DEFINITION")]
    pub definition: AttributeValueDefinitionRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeValueString {
    #[serde(rename = "@THE-VALUE")]
    pub the_value: String,

    #[serde(rename = "DEFINITION")]
    pub definition: AttributeValueDefinitionRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeValueXhtml {
    #[serde(rename = "THE-VALUE", skip_serializing_if = "Option::is_none", default)]
    pub the_value: Option<XhtmlContent>,

    #[serde(rename = "DEFINITION")]
    pub definition: AttributeValueDefinitionRef,
}

/// Stores raw XHTML content as a string. The mapping layer (req1-core)
/// handles conversion to/from Markdown; this crate preserves it as-is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XhtmlContent {
    #[serde(rename = "$text", default)]
    pub content: String,
}

// ---------------------------------------------------------------------------
// Definition reference wrappers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeValueDefinitionRef {
    #[serde(rename = "$value")]
    pub inner: AttrValDefRefInner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttrValDefRefInner {
    #[serde(rename = "ATTRIBUTE-DEFINITION-BOOLEAN-REF")]
    Boolean(String),
    #[serde(rename = "ATTRIBUTE-DEFINITION-DATE-REF")]
    Date(String),
    #[serde(rename = "ATTRIBUTE-DEFINITION-ENUMERATION-REF")]
    Enumeration(String),
    #[serde(rename = "ATTRIBUTE-DEFINITION-INTEGER-REF")]
    Integer(String),
    #[serde(rename = "ATTRIBUTE-DEFINITION-REAL-REF")]
    Real(String),
    #[serde(rename = "ATTRIBUTE-DEFINITION-STRING-REF")]
    Str(String),
    #[serde(rename = "ATTRIBUTE-DEFINITION-XHTML-REF")]
    Xhtml(String),
}
