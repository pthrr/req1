use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReqifError {
    #[error("XML serialization error: {0}")]
    XmlSerialize(#[from] quick_xml::SeError),

    #[error("XML deserialization error: {0}")]
    XmlDeserialize(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("invalid structure: {0}")]
    InvalidStructure(String),

    #[error("unsupported ReqIF version: {0}")]
    UnsupportedVersion(String),

    #[error("missing required field: {0}")]
    MissingField(String),
}
