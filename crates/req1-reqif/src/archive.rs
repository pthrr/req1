use std::io::{BufReader, Read, Seek, Write};

use zip::write::FileOptions;
use zip::CompressionMethod;

use crate::deserialize::from_xml_str;
use crate::error::ReqifError;
use crate::model::ReqIf;
use crate::serialize::to_xml_string;

/// Read a `.reqifz` ZIP archive and parse the first `.reqif` file inside.
pub fn from_reqifz<R: Read + Seek>(reader: R) -> Result<ReqIf, ReqifError> {
    let buf = BufReader::new(reader);
    let mut archive = zip::ZipArchive::new(buf)?;

    let reqif_name = (0..archive.len())
        .find_map(|i| {
            let file = archive.by_index(i).ok()?;
            let name = file.name().to_string();
            if std::path::Path::new(&name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("reqif"))
            {
                Some(name)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            ReqifError::InvalidStructure("no .reqif file found in archive".to_string())
        })?;

    let mut file = archive.by_name(&reqif_name)?;
    let mut xml = String::new();
    let _ = file.read_to_string(&mut xml)?;

    from_xml_str(&xml)
}

/// Write a `ReqIf` document into a `.reqifz` ZIP archive with deflate compression.
pub fn to_reqifz<W: Write + Seek>(
    writer: W,
    doc: &ReqIf,
    filename: &str,
) -> Result<(), ReqifError> {
    let xml = to_xml_string(doc)?;
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);
    zip.start_file(filename, options)?;
    zip.write_all(xml.as_bytes())?;
    let _ = zip.finish()?;
    Ok(())
}
