use std::collections::HashMap;

use req1_reqif::{
    AttributeValue, AttributeValueBoolean, AttributeValueDate, AttributeValueDefinitionRef,
    AttributeValueEnumeration, AttributeValueInteger, AttributeValueReal, AttributeValueString,
    AttributeValueXhtml, AttrValDefRefInner, DatatypeDefinition, DatatypeDefinitionBoolean,
    DatatypeDefinitionDate, DatatypeDefinitionEnumeration, DatatypeDefinitionInteger,
    DatatypeDefinitionReal, DatatypeDefinitionString, DatatypeDefinitionXhtml, EnumValue,
    EnumValueRefs, XhtmlContent,
};
use serde_json::json;

/// Map a `ReqIF` `DatatypeDefinition` variant to the entity `data_type` string.
pub const fn reqif_datatype_to_entity(def: &DatatypeDefinition) -> &'static str {
    match def {
        DatatypeDefinition::Boolean(_) => "boolean",
        DatatypeDefinition::Date(_) => "date",
        DatatypeDefinition::Enumeration(_) => "enum",
        DatatypeDefinition::Integer(_) => "integer",
        DatatypeDefinition::Real(_) => "float",
        DatatypeDefinition::Str(_) => "string",
        DatatypeDefinition::Xhtml(_) => "rich_text",
    }
}

/// Map an entity `data_type` string to a `ReqIF` `DatatypeDefinition` with a generated identifier.
pub fn entity_datatype_to_reqif(data_type: &str, identifier: &str) -> DatatypeDefinition {
    let id = identifier.to_owned();
    match data_type {
        "boolean" => DatatypeDefinition::Boolean(DatatypeDefinitionBoolean {
            identifier: id,
            long_name: None,
            last_change: None,
            desc: None,
        }),
        "date" => DatatypeDefinition::Date(DatatypeDefinitionDate {
            identifier: id,
            long_name: None,
            last_change: None,
            desc: None,
        }),
        "enum" => DatatypeDefinition::Enumeration(DatatypeDefinitionEnumeration {
            identifier: id,
            long_name: None,
            last_change: None,
            desc: None,
            specified_values: None,
        }),
        "integer" => DatatypeDefinition::Integer(DatatypeDefinitionInteger {
            identifier: id,
            long_name: None,
            last_change: None,
            desc: None,
            min: None,
            max: None,
        }),
        "float" => DatatypeDefinition::Real(DatatypeDefinitionReal {
            identifier: id,
            long_name: None,
            last_change: None,
            desc: None,
            min: None,
            max: None,
            accuracy: None,
        }),
        "string" => DatatypeDefinition::Str(DatatypeDefinitionString {
            identifier: id,
            long_name: None,
            last_change: None,
            desc: None,
            max_length: None,
        }),
        _ => DatatypeDefinition::Xhtml(DatatypeDefinitionXhtml {
            identifier: id,
            long_name: None,
            last_change: None,
            desc: None,
        }),
    }
}

/// Convert a `ReqIF` `AttributeValue` to a JSON value for entity storage.
///
/// `enum_values_by_id` maps enum value identifiers to their `long_name` (display string).
pub fn reqif_attr_value_to_json(
    value: &AttributeValue,
    enum_values_by_id: &HashMap<String, String>,
) -> serde_json::Value {
    match value {
        AttributeValue::Boolean(v) => json!(v.the_value),
        AttributeValue::Integer(v) => json!(v.the_value),
        AttributeValue::Real(v) => json!(v.the_value),
        AttributeValue::Str(v) => json!(v.the_value),
        AttributeValue::Date(v) => json!(v.the_value),
        AttributeValue::Xhtml(v) => {
            let text = v
                .the_value
                .as_ref()
                .map_or(String::new(), |xhtml| xhtml.content.clone());
            json!(text)
        }
        AttributeValue::Enumeration(v) => {
            let names: Vec<String> = v
                .values
                .as_ref()
                .map(|refs| {
                    refs.refs
                        .iter()
                        .filter_map(|r| enum_values_by_id.get(r).cloned())
                        .collect()
                })
                .unwrap_or_default();
            if names.len() == 1 {
                names
                    .first()
                    .map_or_else(|| json!([]), |n| json!(n))
            } else {
                json!(names)
            }
        }
    }
}

/// Extract the attribute definition reference ID from an `AttributeValue`.
pub fn attr_value_def_ref(value: &AttributeValue) -> &str {
    match value {
        AttributeValue::Boolean(v) => def_ref_id(&v.definition),
        AttributeValue::Date(v) => def_ref_id(&v.definition),
        AttributeValue::Enumeration(v) => def_ref_id(&v.definition),
        AttributeValue::Integer(v) => def_ref_id(&v.definition),
        AttributeValue::Real(v) => def_ref_id(&v.definition),
        AttributeValue::Str(v) => def_ref_id(&v.definition),
        AttributeValue::Xhtml(v) => def_ref_id(&v.definition),
    }
}

fn def_ref_id(def: &AttributeValueDefinitionRef) -> &str {
    match &def.inner {
        AttrValDefRefInner::Boolean(id)
        | AttrValDefRefInner::Date(id)
        | AttrValDefRefInner::Enumeration(id)
        | AttrValDefRefInner::Integer(id)
        | AttrValDefRefInner::Real(id)
        | AttrValDefRefInner::Str(id)
        | AttrValDefRefInner::Xhtml(id) => id,
    }
}

/// Convert a JSON value back to a `ReqIF` `AttributeValue`.
///
/// - `attr_def_id`: the `ReqIF` identifier for the attribute definition
/// - `data_type`: the entity `data_type` string
/// - `enum_name_to_id`: maps enum `long_name` to `ReqIF` enum value `identifier` (for export)
#[allow(clippy::too_many_lines)]
pub fn json_to_reqif_attr_value(
    value: &serde_json::Value,
    attr_def_id: &str,
    data_type: &str,
    enum_name_to_id: &HashMap<String, String>,
) -> Option<AttributeValue> {
    let def_id = attr_def_id.to_owned();
    match data_type {
        "boolean" => {
            let b = value.as_bool()?;
            Some(AttributeValue::Boolean(AttributeValueBoolean {
                the_value: b,
                definition: AttributeValueDefinitionRef {
                    inner: AttrValDefRefInner::Boolean(def_id),
                },
            }))
        }
        "integer" => {
            let n = value.as_i64()?;
            Some(AttributeValue::Integer(AttributeValueInteger {
                the_value: n,
                definition: AttributeValueDefinitionRef {
                    inner: AttrValDefRefInner::Integer(def_id),
                },
            }))
        }
        "float" => {
            let n = value.as_f64()?;
            Some(AttributeValue::Real(AttributeValueReal {
                the_value: n,
                definition: AttributeValueDefinitionRef {
                    inner: AttrValDefRefInner::Real(def_id),
                },
            }))
        }
        "string" => {
            let s = value.as_str()?;
            Some(AttributeValue::Str(AttributeValueString {
                the_value: s.to_owned(),
                definition: AttributeValueDefinitionRef {
                    inner: AttrValDefRefInner::Str(def_id),
                },
            }))
        }
        "date" => {
            let s = value.as_str()?;
            Some(AttributeValue::Date(AttributeValueDate {
                the_value: s.to_owned(),
                definition: AttributeValueDefinitionRef {
                    inner: AttrValDefRefInner::Date(def_id),
                },
            }))
        }
        "rich_text" => {
            let s = value.as_str().unwrap_or_default();
            Some(AttributeValue::Xhtml(AttributeValueXhtml {
                the_value: Some(XhtmlContent {
                    content: s.to_owned(),
                }),
                definition: AttributeValueDefinitionRef {
                    inner: AttrValDefRefInner::Xhtml(def_id),
                },
            }))
        }
        "enum" => {
            let refs = match value {
                serde_json::Value::String(s) => {
                    vec![enum_name_to_id.get(s.as_str()).cloned().unwrap_or_default()]
                }
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| enum_name_to_id.get(s).cloned().unwrap_or_default())
                    .collect(),
                _ => return None,
            };
            Some(AttributeValue::Enumeration(AttributeValueEnumeration {
                values: Some(EnumValueRefs { refs }),
                definition: AttributeValueDefinitionRef {
                    inner: AttrValDefRefInner::Enumeration(def_id),
                },
            }))
        }
        _ => None,
    }
}

/// Extract the identifier from a `DatatypeDefinition`.
pub fn datatype_identifier(def: &DatatypeDefinition) -> &str {
    match def {
        DatatypeDefinition::Boolean(d) => &d.identifier,
        DatatypeDefinition::Date(d) => &d.identifier,
        DatatypeDefinition::Enumeration(d) => &d.identifier,
        DatatypeDefinition::Integer(d) => &d.identifier,
        DatatypeDefinition::Real(d) => &d.identifier,
        DatatypeDefinition::Str(d) => &d.identifier,
        DatatypeDefinition::Xhtml(d) => &d.identifier,
    }
}

/// Extract enum values from a `DatatypeDefinitionEnumeration` if present, keyed by identifier.
pub fn extract_enum_values(datatypes: &[DatatypeDefinition]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for dt in datatypes {
        if let DatatypeDefinition::Enumeration(e) = dt
            && let Some(sv) = &e.specified_values
        {
            for ev in &sv.values {
                let name = ev.long_name.clone().unwrap_or_else(|| ev.identifier.clone());
                let _ = map.insert(ev.identifier.clone(), name);
            }
        }
    }
    map
}

/// Build enum values JSON array from `ReqIF` `EnumValue` items.
pub fn enum_values_to_json(values: &[EnumValue]) -> serde_json::Value {
    let names: Vec<String> = values
        .iter()
        .map(|v| v.long_name.clone().unwrap_or_else(|| v.identifier.clone()))
        .collect();
    json!(names)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use req1_reqif::{
        AttributeValueBoolean, AttributeValueDate, AttributeValueDefinitionRef,
        AttributeValueEnumeration, AttributeValueInteger, AttributeValueReal,
        AttributeValueString, AttributeValueXhtml, AttrValDefRefInner, DatatypeDefinition,
        DatatypeDefinitionBoolean, DatatypeDefinitionDate, DatatypeDefinitionEnumeration,
        DatatypeDefinitionInteger, DatatypeDefinitionReal, DatatypeDefinitionString,
        DatatypeDefinitionXhtml, EnumValue, EnumValueRefs, SpecifiedValues, XhtmlContent,
    };

    #[test]
    fn test_reqif_datatype_to_entity() {
        let cases: Vec<(DatatypeDefinition, &str)> = vec![
            (
                DatatypeDefinition::Boolean(DatatypeDefinitionBoolean {
                    identifier: "d1".into(),
                    long_name: None,
                    last_change: None,
                    desc: None,
                }),
                "boolean",
            ),
            (
                DatatypeDefinition::Date(DatatypeDefinitionDate {
                    identifier: "d2".into(),
                    long_name: None,
                    last_change: None,
                    desc: None,
                }),
                "date",
            ),
            (
                DatatypeDefinition::Enumeration(DatatypeDefinitionEnumeration {
                    identifier: "d3".into(),
                    long_name: None,
                    last_change: None,
                    desc: None,
                    specified_values: None,
                }),
                "enum",
            ),
            (
                DatatypeDefinition::Integer(DatatypeDefinitionInteger {
                    identifier: "d4".into(),
                    long_name: None,
                    last_change: None,
                    desc: None,
                    min: None,
                    max: None,
                }),
                "integer",
            ),
            (
                DatatypeDefinition::Real(DatatypeDefinitionReal {
                    identifier: "d5".into(),
                    long_name: None,
                    last_change: None,
                    desc: None,
                    min: None,
                    max: None,
                    accuracy: None,
                }),
                "float",
            ),
            (
                DatatypeDefinition::Str(DatatypeDefinitionString {
                    identifier: "d6".into(),
                    long_name: None,
                    last_change: None,
                    desc: None,
                    max_length: None,
                }),
                "string",
            ),
            (
                DatatypeDefinition::Xhtml(DatatypeDefinitionXhtml {
                    identifier: "d7".into(),
                    long_name: None,
                    last_change: None,
                    desc: None,
                }),
                "rich_text",
            ),
        ];

        for (def, expected) in &cases {
            assert_eq!(reqif_datatype_to_entity(def), *expected);
        }
    }

    #[test]
    fn test_entity_datatype_to_reqif_roundtrip() {
        let types = [
            "boolean", "date", "enum", "integer", "float", "string", "rich_text",
        ];
        for t in &types {
            let def = entity_datatype_to_reqif(t, &format!("test-{t}"));
            assert_eq!(reqif_datatype_to_entity(&def), *t);
        }
    }

    #[test]
    fn test_attr_value_to_json_boolean() {
        let val = AttributeValue::Boolean(AttributeValueBoolean {
            the_value: true,
            definition: AttributeValueDefinitionRef {
                inner: AttrValDefRefInner::Boolean("ad-1".into()),
            },
        });
        assert_eq!(reqif_attr_value_to_json(&val, &HashMap::new()), json!(true));
    }

    #[test]
    fn test_attr_value_to_json_integer() {
        let val = AttributeValue::Integer(AttributeValueInteger {
            the_value: 42,
            definition: AttributeValueDefinitionRef {
                inner: AttrValDefRefInner::Integer("ad-1".into()),
            },
        });
        assert_eq!(reqif_attr_value_to_json(&val, &HashMap::new()), json!(42));
    }

    #[test]
    fn test_attr_value_to_json_real() {
        let val = AttributeValue::Real(AttributeValueReal {
            the_value: 95.5,
            definition: AttributeValueDefinitionRef {
                inner: AttrValDefRefInner::Real("ad-1".into()),
            },
        });
        assert_eq!(
            reqif_attr_value_to_json(&val, &HashMap::new()),
            json!(95.5)
        );
    }

    #[test]
    fn test_attr_value_to_json_string() {
        let val = AttributeValue::Str(AttributeValueString {
            the_value: "hello".into(),
            definition: AttributeValueDefinitionRef {
                inner: AttrValDefRefInner::Str("ad-1".into()),
            },
        });
        assert_eq!(
            reqif_attr_value_to_json(&val, &HashMap::new()),
            json!("hello")
        );
    }

    #[test]
    fn test_attr_value_to_json_date() {
        let val = AttributeValue::Date(AttributeValueDate {
            the_value: "2024-01-01T00:00:00Z".into(),
            definition: AttributeValueDefinitionRef {
                inner: AttrValDefRefInner::Date("ad-1".into()),
            },
        });
        assert_eq!(
            reqif_attr_value_to_json(&val, &HashMap::new()),
            json!("2024-01-01T00:00:00Z")
        );
    }

    #[test]
    fn test_attr_value_to_json_xhtml() {
        let val = AttributeValue::Xhtml(AttributeValueXhtml {
            the_value: Some(XhtmlContent {
                content: "<b>Bold</b>".into(),
            }),
            definition: AttributeValueDefinitionRef {
                inner: AttrValDefRefInner::Xhtml("ad-1".into()),
            },
        });
        assert_eq!(
            reqif_attr_value_to_json(&val, &HashMap::new()),
            json!("<b>Bold</b>")
        );
    }

    #[test]
    fn test_attr_value_to_json_enum_single() {
        let mut enum_map = HashMap::new();
        let _ = enum_map.insert("ev-1".to_owned(), "Low".to_owned());
        let _ = enum_map.insert("ev-2".to_owned(), "High".to_owned());

        let val = AttributeValue::Enumeration(AttributeValueEnumeration {
            values: Some(EnumValueRefs {
                refs: vec!["ev-1".into()],
            }),
            definition: AttributeValueDefinitionRef {
                inner: AttrValDefRefInner::Enumeration("ad-1".into()),
            },
        });
        assert_eq!(reqif_attr_value_to_json(&val, &enum_map), json!("Low"));
    }

    #[test]
    fn test_attr_value_to_json_enum_multi() {
        let mut enum_map = HashMap::new();
        let _ = enum_map.insert("ev-1".to_owned(), "Low".to_owned());
        let _ = enum_map.insert("ev-2".to_owned(), "High".to_owned());

        let val = AttributeValue::Enumeration(AttributeValueEnumeration {
            values: Some(EnumValueRefs {
                refs: vec!["ev-1".into(), "ev-2".into()],
            }),
            definition: AttributeValueDefinitionRef {
                inner: AttrValDefRefInner::Enumeration("ad-1".into()),
            },
        });
        assert_eq!(
            reqif_attr_value_to_json(&val, &enum_map),
            json!(["Low", "High"])
        );
    }

    #[test]
    fn test_json_to_reqif_boolean() {
        let val = json!(true);
        let result =
            json_to_reqif_attr_value(&val, "ad-1", "boolean", &HashMap::new()).expect("boolean");
        assert!(matches!(result, AttributeValue::Boolean(b) if b.the_value));
    }

    #[test]
    fn test_json_to_reqif_integer() {
        let val = json!(99);
        let result =
            json_to_reqif_attr_value(&val, "ad-1", "integer", &HashMap::new()).expect("integer");
        assert!(matches!(result, AttributeValue::Integer(i) if i.the_value == 99));
    }

    #[test]
    fn test_json_to_reqif_enum() {
        let mut name_to_id = HashMap::new();
        let _ = name_to_id.insert("Low".to_owned(), "ev-1".to_owned());

        let val = json!("Low");
        let result = json_to_reqif_attr_value(&val, "ad-1", "enum", &name_to_id).expect("enum");
        if let AttributeValue::Enumeration(e) = result {
            assert_eq!(e.values.expect("values").refs, vec!["ev-1".to_owned()]);
        } else {
            panic!("expected enumeration");
        }
    }

    #[test]
    fn test_extract_enum_values() {
        let dts = vec![DatatypeDefinition::Enumeration(
            DatatypeDefinitionEnumeration {
                identifier: "dt-enum".into(),
                long_name: None,
                last_change: None,
                desc: None,
                specified_values: Some(SpecifiedValues {
                    values: vec![
                        EnumValue {
                            identifier: "ev-1".into(),
                            long_name: Some("Low".into()),
                            last_change: None,
                            desc: None,
                            properties: None,
                        },
                        EnumValue {
                            identifier: "ev-2".into(),
                            long_name: None,
                            last_change: None,
                            desc: None,
                            properties: None,
                        },
                    ],
                }),
            },
        )];

        let map = extract_enum_values(&dts);
        assert_eq!(map.get("ev-1").expect("ev-1"), "Low");
        // Falls back to identifier when long_name is None
        assert_eq!(map.get("ev-2").expect("ev-2"), "ev-2");
    }
}
