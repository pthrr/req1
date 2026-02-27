#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use pretty_assertions::assert_eq;
use req1_reqif::{from_xml_str, to_xml_string};

fn roundtrip(xml: &str) {
    let doc = from_xml_str(xml).expect("parse failed");
    let serialized = to_xml_string(&doc).expect("serialize failed");
    let reparsed = from_xml_str(&serialized).expect("re-parse failed");
    assert_eq!(doc, reparsed);
}

#[test]
fn roundtrip_minimal() {
    let xml = include_str!("fixtures/minimal.reqif");
    roundtrip(xml);
}

#[test]
fn roundtrip_all_types() {
    let xml = include_str!("fixtures/all_types.reqif");
    roundtrip(xml);
}

#[test]
fn parse_minimal_header() {
    let xml = include_str!("fixtures/minimal.reqif");
    let doc = from_xml_str(xml).unwrap();
    let hdr = &doc.the_header.req_if_header;
    assert_eq!(hdr.identifier, "header-1");
    assert_eq!(hdr.title.as_deref(), Some("Minimal ReqIF"));
    assert_eq!(hdr.req_if_version.as_deref(), Some("1.2"));
}

#[test]
fn parse_minimal_spec_objects() {
    let xml = include_str!("fixtures/minimal.reqif");
    let doc = from_xml_str(xml).unwrap();
    let objects = &doc.core_content.req_if_content.spec_objects.as_ref().unwrap().objects;
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].identifier, "so-1");
    assert_eq!(objects[0].long_name.as_deref(), Some("REQ-001"));
}

#[test]
fn parse_minimal_hierarchy() {
    let xml = include_str!("fixtures/minimal.reqif");
    let doc = from_xml_str(xml).unwrap();
    let specs = &doc.core_content.req_if_content.specifications.as_ref().unwrap().specifications;
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].identifier, "spec-1");
    let children = specs[0].children.as_ref().unwrap();
    assert_eq!(children.hierarchies.len(), 1);
    assert_eq!(children.hierarchies[0].object.spec_object_ref, "so-1");
}

#[test]
fn parse_all_types_datatypes() {
    let xml = include_str!("fixtures/all_types.reqif");
    let doc = from_xml_str(xml).unwrap();
    let dt = doc.core_content.req_if_content.datatypes.as_ref().unwrap();
    assert_eq!(dt.definitions.len(), 7);
}

#[test]
fn parse_all_types_attribute_values() {
    use req1_reqif::model::spec_objects::AttributeValue;

    let xml = include_str!("fixtures/all_types.reqif");
    let doc = from_xml_str(xml).unwrap();
    let obj = &doc.core_content.req_if_content.spec_objects.as_ref().unwrap().objects[0];
    let vals = &obj.values.as_ref().unwrap().values;
    assert_eq!(vals.len(), 7);

    // Check boolean
    assert!(matches!(&vals[0], AttributeValue::Boolean(v) if v.the_value));
    // Check integer
    assert!(matches!(&vals[3], AttributeValue::Integer(v) if v.the_value == 42));
    // Check real
    assert!(matches!(&vals[4], AttributeValue::Real(v) if (v.the_value - 95.5).abs() < f64::EPSILON));
    // Check string
    assert!(
        matches!(&vals[5], AttributeValue::Str(v) if v.the_value == "System shall handle all types.")
    );
}

#[test]
fn parse_all_types_relations() {
    let xml = include_str!("fixtures/all_types.reqif");
    let doc = from_xml_str(xml).unwrap();
    let rels = &doc.core_content.req_if_content.spec_relations.as_ref().unwrap().relations;
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0].source.spec_object_ref, "so-1");
    assert_eq!(rels[0].target.spec_object_ref, "so-2");
}

#[test]
fn parse_all_types_nested_hierarchy() {
    let xml = include_str!("fixtures/all_types.reqif");
    let doc = from_xml_str(xml).unwrap();
    let spec = &doc.core_content.req_if_content.specifications.as_ref().unwrap().specifications[0];
    let top = &spec.children.as_ref().unwrap().hierarchies;
    assert_eq!(top.len(), 2);
    // First hierarchy has nested child
    let nested = top[0].children.as_ref().unwrap();
    assert_eq!(nested.hierarchies.len(), 1);
    assert_eq!(nested.hierarchies[0].object.spec_object_ref, "so-3");
}

#[test]
fn parse_all_types_relation_groups() {
    let xml = include_str!("fixtures/all_types.reqif");
    let doc = from_xml_str(xml).unwrap();
    let groups = &doc
        .core_content
        .req_if_content
        .spec_relation_groups
        .as_ref()
        .unwrap()
        .groups;
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].identifier, "rg-1");
    assert_eq!(groups[0].source_specification.value, "spec-1");
}

#[test]
fn unsupported_version_rejected() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<REQ-IF>
  <THE-HEADER>
    <REQ-IF-HEADER IDENTIFIER="h1">
      <REQ-IF-VERSION>2.0</REQ-IF-VERSION>
    </REQ-IF-HEADER>
  </THE-HEADER>
  <CORE-CONTENT>
    <REQ-IF-CONTENT/>
  </CORE-CONTENT>
</REQ-IF>"#;
    let err = from_xml_str(xml).unwrap_err();
    assert!(err.to_string().contains("2.0"));
}
