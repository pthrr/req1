use std::io::Write;

use crate::error::ReqifError;
use crate::model::ReqIf;

/// Serialize a `ReqIf` document to an XML string with declaration and 2-space indent.
pub fn to_xml_string(doc: &ReqIf) -> Result<String, ReqifError> {
    let mut buffer = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let mut ser = quick_xml::se::Serializer::new(&mut buffer);
    let _ = ser.indent(' ', 2);
    let _ = serde::Serialize::serialize(doc, ser)?;
    Ok(buffer)
}

/// Serialize a `ReqIf` document and write the XML bytes to a writer.
pub fn to_xml_writer<W: Write>(mut writer: W, doc: &ReqIf) -> Result<(), ReqifError> {
    let xml = to_xml_string(doc)?;
    writer.write_all(xml.as_bytes())?;
    Ok(())
}
