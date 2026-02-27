pub mod archive;
pub mod deserialize;
pub mod error;
pub mod model;
pub mod serialize;

pub use archive::{from_reqifz, to_reqifz};
pub use deserialize::{from_xml_reader, from_xml_str};
pub use error::ReqifError;
pub use model::*;
pub use serialize::{to_xml_string, to_xml_writer};

/// Builder for constructing `ReqIf` documents with sensible defaults.
#[must_use]
pub struct ReqIfBuilder {
    identifier: String,
    title: String,
    comment: Option<String>,
    repository_id: Option<String>,
    datatypes: Option<Datatypes>,
    spec_types: Option<SpecTypes>,
    spec_objects: Option<SpecObjects>,
    spec_relations: Option<SpecRelations>,
    specifications: Option<Specifications>,
    spec_relation_groups: Option<SpecRelationGroups>,
}

impl ReqIfBuilder {
    pub fn new(identifier: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            title: title.into(),
            comment: None,
            repository_id: None,
            datatypes: None,
            spec_types: None,
            spec_objects: None,
            spec_relations: None,
            specifications: None,
            spec_relation_groups: None,
        }
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn repository_id(mut self, id: impl Into<String>) -> Self {
        self.repository_id = Some(id.into());
        self
    }

    pub fn datatypes(mut self, datatypes: Datatypes) -> Self {
        self.datatypes = Some(datatypes);
        self
    }

    pub fn spec_types(mut self, spec_types: SpecTypes) -> Self {
        self.spec_types = Some(spec_types);
        self
    }

    pub fn spec_objects(mut self, spec_objects: SpecObjects) -> Self {
        self.spec_objects = Some(spec_objects);
        self
    }

    pub fn spec_relations(mut self, spec_relations: SpecRelations) -> Self {
        self.spec_relations = Some(spec_relations);
        self
    }

    pub fn specifications(mut self, specifications: Specifications) -> Self {
        self.specifications = Some(specifications);
        self
    }

    pub fn spec_relation_groups(mut self, groups: SpecRelationGroups) -> Self {
        self.spec_relation_groups = Some(groups);
        self
    }

    pub fn build(self) -> ReqIf {
        let now = chrono::Utc::now().to_rfc3339();
        ReqIf {
            xmlns: Some("http://www.omg.org/spec/ReqIF/20110401/reqif.xsd".to_string()),
            the_header: TheHeader {
                req_if_header: ReqIfHeader {
                    identifier: self.identifier,
                    comment: self.comment,
                    creation_time: Some(now),
                    repository_id: self.repository_id,
                    req_if_tool_id: Some("req1".to_string()),
                    req_if_version: Some("1.2".to_string()),
                    source_tool_id: Some("req1".to_string()),
                    title: Some(self.title),
                },
            },
            core_content: CoreContent {
                req_if_content: ReqIfContent {
                    datatypes: self.datatypes,
                    spec_types: self.spec_types,
                    spec_objects: self.spec_objects,
                    spec_relations: self.spec_relations,
                    specifications: self.specifications,
                    spec_relation_groups: self.spec_relation_groups,
                },
            },
        }
    }
}
