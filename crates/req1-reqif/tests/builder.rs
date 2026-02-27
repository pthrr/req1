#![allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::too_many_lines)]

use req1_reqif::model::datatypes::*;
use req1_reqif::model::spec_hierarchy::*;
use req1_reqif::model::spec_objects::*;
use req1_reqif::model::spec_types::*;
use req1_reqif::{ReqIfBuilder, from_xml_str, to_xml_string};

#[test]
fn builder_minimal() {
    let doc = ReqIfBuilder::new("test-id", "Test Document").build();
    assert_eq!(doc.the_header.req_if_header.identifier, "test-id");
    assert_eq!(
        doc.the_header.req_if_header.title.as_deref(),
        Some("Test Document")
    );
    assert_eq!(
        doc.the_header.req_if_header.req_if_version.as_deref(),
        Some("1.2")
    );
    assert_eq!(
        doc.the_header.req_if_header.req_if_tool_id.as_deref(),
        Some("req1")
    );
    assert!(doc.the_header.req_if_header.creation_time.is_some());
}

#[test]
fn builder_roundtrip() {
    let doc = ReqIfBuilder::new("builder-test", "Builder Test")
        .comment("Built by test")
        .repository_id("repo-1")
        .datatypes(Datatypes {
            definitions: vec![DatatypeDefinition::Str(DatatypeDefinitionString {
                identifier: "dt-str".into(),
                long_name: Some("Text".into()),
                last_change: None,
                desc: None,
                max_length: Some(4096),
            })],
        })
        .spec_types(SpecTypes {
            types: vec![
                SpecType::SpecObjectType(SpecObjectType {
                    identifier: "sot-1".into(),
                    long_name: Some("Requirement".into()),
                    last_change: None,
                    desc: None,
                    spec_attributes: Some(SpecAttributes {
                        definitions: vec![AttributeDefinition::Str(AttributeDefinitionString {
                            identifier: "ad-str".into(),
                            long_name: Some("Text".into()),
                            last_change: None,
                            desc: None,
                            is_editable: Some(true),
                            datatype_ref: AttrDefTypeRef {
                                inner: AttrDefTypeRefInner::Str("dt-str".into()),
                            },
                        })],
                    }),
                }),
                SpecType::SpecificationType(SpecificationType {
                    identifier: "st-1".into(),
                    long_name: Some("Doc".into()),
                    last_change: None,
                    desc: None,
                    spec_attributes: None,
                }),
            ],
        })
        .spec_objects(SpecObjects {
            objects: vec![SpecObject {
                identifier: "so-1".into(),
                long_name: Some("REQ-001".into()),
                last_change: None,
                desc: None,
                type_ref: SpecObjectTypeRef {
                    value: "sot-1".into(),
                },
                values: Some(AttributeValues {
                    values: vec![AttributeValue::Str(AttributeValueString {
                        the_value: "System shall work.".into(),
                        definition: AttributeValueDefinitionRef {
                            inner: AttrValDefRefInner::Str("ad-str".into()),
                        },
                    })],
                }),
            }],
        })
        .specifications(Specifications {
            specifications: vec![Specification {
                identifier: "spec-1".into(),
                long_name: Some("Test Spec".into()),
                last_change: None,
                desc: None,
                type_ref: SpecificationTypeRef {
                    value: "st-1".into(),
                },
                values: None,
                children: Some(SpecHierarchyChildren {
                    hierarchies: vec![SpecHierarchy {
                        identifier: "sh-1".into(),
                        long_name: None,
                        last_change: None,
                        desc: None,
                        is_table_internal: None,
                        object: SpecHierarchyObjectRef {
                            spec_object_ref: "so-1".into(),
                        },
                        children: None,
                    }],
                }),
            }],
        })
        .build();

    let xml = to_xml_string(&doc).expect("serialize");
    let reparsed = from_xml_str(&xml).expect("re-parse");

    assert_eq!(reparsed.the_header.req_if_header.identifier, "builder-test");
    assert_eq!(
        reparsed.the_header.req_if_header.comment.as_deref(),
        Some("Built by test")
    );

    let objects = reparsed
        .core_content
        .req_if_content
        .spec_objects
        .as_ref()
        .unwrap();
    assert_eq!(objects.objects.len(), 1);
    assert_eq!(objects.objects[0].identifier, "so-1");
}
