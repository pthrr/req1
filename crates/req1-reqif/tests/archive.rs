#![allow(clippy::unwrap_used)]

use std::io::Cursor;

use req1_reqif::{ReqIfBuilder, from_reqifz, to_reqifz};

#[test]
fn reqifz_roundtrip() {
    let doc = ReqIfBuilder::new("zip-test", "ZIP Test").build();

    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);
    to_reqifz(cursor, &doc, "test.reqif").expect("write reqifz");

    let reader = Cursor::new(&buf);
    let reparsed = from_reqifz(reader).expect("read reqifz");

    assert_eq!(reparsed.the_header.req_if_header.identifier, "zip-test");
    assert_eq!(
        reparsed.the_header.req_if_header.title.as_deref(),
        Some("ZIP Test")
    );
}

#[test]
fn reqifz_no_reqif_file() {
    // Create a ZIP with no .reqif file
    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options =
            zip::write::FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("readme.txt", options).unwrap();
        std::io::Write::write_all(&mut zip, b"not a reqif").unwrap();
        let _ = zip.finish().unwrap();
    }

    let reader = Cursor::new(&buf);
    let err = from_reqifz(reader).expect_err("should fail with no .reqif file");
    assert!(err.to_string().contains("no .reqif file"));
}
