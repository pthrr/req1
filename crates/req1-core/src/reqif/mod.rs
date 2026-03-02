pub mod export;
pub mod import;
mod type_map;

use uuid::Uuid;

/// Result of a `ReqIF` import operation.
#[derive(Debug)]
pub struct ImportResult {
    pub module_id: Uuid,
    pub objects_created: usize,
    pub links_created: usize,
    pub attribute_definitions_created: usize,
    pub object_types_created: usize,
    pub link_types_created: usize,
}

/// Result of a `ReqIF` export operation.
#[derive(Debug)]
pub struct ExportResult {
    pub document: req1_reqif::ReqIf,
    pub objects_exported: usize,
    pub links_exported: usize,
}
