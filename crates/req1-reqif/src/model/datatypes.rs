use serde::{Deserialize, Serialize};

use super::common::identifiable_struct;

// ---------------------------------------------------------------------------
// Container
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "DATATYPES")]
pub struct Datatypes {
    #[serde(rename = "$value", default)]
    pub definitions: Vec<DatatypeDefinition>,
}

// ---------------------------------------------------------------------------
// Enum dispatch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DatatypeDefinition {
    #[serde(rename = "DATATYPE-DEFINITION-BOOLEAN")]
    Boolean(DatatypeDefinitionBoolean),
    #[serde(rename = "DATATYPE-DEFINITION-DATE")]
    Date(DatatypeDefinitionDate),
    #[serde(rename = "DATATYPE-DEFINITION-ENUMERATION")]
    Enumeration(DatatypeDefinitionEnumeration),
    #[serde(rename = "DATATYPE-DEFINITION-INTEGER")]
    Integer(DatatypeDefinitionInteger),
    #[serde(rename = "DATATYPE-DEFINITION-REAL")]
    Real(DatatypeDefinitionReal),
    #[serde(rename = "DATATYPE-DEFINITION-STRING")]
    Str(DatatypeDefinitionString),
    #[serde(rename = "DATATYPE-DEFINITION-XHTML")]
    Xhtml(DatatypeDefinitionXhtml),
}

// ---------------------------------------------------------------------------
// Variant structs
// ---------------------------------------------------------------------------

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DatatypeDefinitionBoolean {
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DatatypeDefinitionDate {
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DatatypeDefinitionEnumeration {
        #[serde(rename = "SPECIFIED-VALUES", skip_serializing_if = "Option::is_none", default)]
        specified_values: Option<SpecifiedValues>,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "SPECIFIED-VALUES")]
pub struct SpecifiedValues {
    #[serde(rename = "ENUM-VALUE", default)]
    pub values: Vec<EnumValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "ENUM-VALUE")]
pub struct EnumValue {
    #[serde(rename = "@IDENTIFIER")]
    pub identifier: String,

    #[serde(rename = "@LONG-NAME", skip_serializing_if = "Option::is_none", default)]
    pub long_name: Option<String>,

    #[serde(rename = "@LAST-CHANGE", skip_serializing_if = "Option::is_none", default)]
    pub last_change: Option<String>,

    #[serde(rename = "@DESC", skip_serializing_if = "Option::is_none", default)]
    pub desc: Option<String>,

    #[serde(rename = "PROPERTIES", skip_serializing_if = "Option::is_none", default)]
    pub properties: Option<EnumValueProperties>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "PROPERTIES")]
pub struct EnumValueProperties {
    #[serde(rename = "EMBEDDED-VALUE")]
    pub embedded_value: EmbeddedValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "EMBEDDED-VALUE")]
pub struct EmbeddedValue {
    #[serde(rename = "@KEY")]
    pub key: i64,

    #[serde(rename = "@OTHER-CONTENT", skip_serializing_if = "Option::is_none", default)]
    pub other_content: Option<String>,
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DatatypeDefinitionInteger {
        #[serde(rename = "@MIN", skip_serializing_if = "Option::is_none", default)]
        min: Option<i64>,

        #[serde(rename = "@MAX", skip_serializing_if = "Option::is_none", default)]
        max: Option<i64>,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DatatypeDefinitionReal {
        #[serde(rename = "@MIN", skip_serializing_if = "Option::is_none", default)]
        min: Option<f64>,

        #[serde(rename = "@MAX", skip_serializing_if = "Option::is_none", default)]
        max: Option<f64>,

        #[serde(rename = "@ACCURACY", skip_serializing_if = "Option::is_none", default)]
        accuracy: Option<i32>,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DatatypeDefinitionString {
        #[serde(rename = "@MAX-LENGTH", skip_serializing_if = "Option::is_none", default)]
        max_length: Option<i32>,
    }
}

identifiable_struct! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DatatypeDefinitionXhtml {
    }
}
