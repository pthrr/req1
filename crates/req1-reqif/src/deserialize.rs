use std::io::BufRead;

use crate::error::ReqifError;
use crate::model::ReqIf;

const SUPPORTED_VERSIONS: &[&str] = &["1.0", "1.0.1", "1.1", "1.2"];

/// Deserialize a `ReqIf` document from an XML string.
pub fn from_xml_str(xml: &str) -> Result<ReqIf, ReqifError> {
    let doc: ReqIf =
        quick_xml::de::from_str(xml).map_err(|e| ReqifError::XmlDeserialize(e.to_string()))?;
    validate_version(&doc)?;
    Ok(doc)
}

/// Deserialize a `ReqIf` document from a buffered reader.
pub fn from_xml_reader<R: BufRead>(reader: R) -> Result<ReqIf, ReqifError> {
    let doc: ReqIf =
        quick_xml::de::from_reader(reader).map_err(|e| ReqifError::XmlDeserialize(e.to_string()))?;
    validate_version(&doc)?;
    Ok(doc)
}

fn validate_version(doc: &ReqIf) -> Result<(), ReqifError> {
    if let Some(version) = &doc.the_header.req_if_header.req_if_version
        && !SUPPORTED_VERSIONS.contains(&version.as_str())
    {
        return Err(ReqifError::UnsupportedVersion(version.clone()));
    }
    Ok(())
}
