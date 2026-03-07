use std::io::{Cursor, Read as _};

use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use entity::object;

use crate::error::CoreError;
use crate::service::object::{CreateObjectInput, ObjectService, UpdateObjectInput};

#[derive(Debug, Serialize, ToSchema)]
pub struct DiscoveredStyle {
    pub style_id: String,
    pub sample_text: String,
    pub count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocxPreviewResult {
    pub styles: Vec<DiscoveredStyle>,
    pub paragraph_count: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StyleMapping {
    pub style_id: String,
    pub classification: String,
    pub is_heading: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DocxImportInput {
    #[serde(default)]
    pub style_mappings: Vec<StyleMapping>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocxImportResult {
    pub objects_created: usize,
    pub objects_updated: usize,
    pub paragraphs_skipped: usize,
}

struct ParsedParagraph {
    style_id: String,
    text: String,
    bookmark_id: Option<String>,
}

pub struct DocxImportService;

impl DocxImportService {
    #[allow(clippy::unused_async)]
    pub async fn preview_docx(
        _db: &impl ConnectionTrait,
        _module_id: Uuid,
        data: &[u8],
    ) -> Result<DocxPreviewResult, CoreError> {
        let paragraphs = parse_docx_paragraphs(data)?;

        let mut style_counts: std::collections::HashMap<String, (String, usize)> =
            std::collections::HashMap::new();

        for para in &paragraphs {
            let entry = style_counts
                .entry(para.style_id.clone())
                .or_insert_with(|| (String::new(), 0));
            if entry.0.is_empty() && !para.text.is_empty() {
                entry.0.clone_from(&para.text);
            }
            entry.1 += 1;
        }

        let styles = style_counts
            .into_iter()
            .map(|(style_id, (sample_text, count))| DiscoveredStyle {
                style_id,
                sample_text: sample_text.chars().take(100).collect(),
                count,
            })
            .collect();

        Ok(DocxPreviewResult {
            styles,
            paragraph_count: paragraphs.len(),
        })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn import_docx(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        data: &[u8],
        input: DocxImportInput,
    ) -> Result<DocxImportResult, CoreError> {
        // Verify module exists
        let _module = entity::module::Entity::find_by_id(module_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found(format!("module {module_id} not found")))?;

        let paragraphs = parse_docx_paragraphs(data)?;

        // Build style mapping lookup
        let mapping_by_style: std::collections::HashMap<String, &StyleMapping> = input
            .style_mappings
            .iter()
            .map(|m| (m.style_id.clone(), m))
            .collect();

        // Fetch existing objects with docx_source_id for round-trip
        let existing_by_docx_id: std::collections::HashMap<String, object::Model> =
            object::Entity::find()
                .filter(object::Column::ModuleId.eq(module_id))
                .filter(object::Column::DeletedAt.is_null())
                .all(db)
                .await?
                .into_iter()
                .filter_map(|o| o.docx_source_id.clone().map(|docx_id| (docx_id, o)))
                .collect();

        let mut objects_created: usize = 0;
        let mut objects_updated: usize = 0;
        let mut paragraphs_skipped: usize = 0;

        let mut level_stack: Vec<(usize, Uuid)> = Vec::new();
        let mut pending_heading: Option<(String, String, Option<String>)> = None;
        let mut body_parts: Vec<String> = Vec::new();

        let heading_styles: std::collections::HashSet<&str> = mapping_by_style
            .iter()
            .filter(|(_, m)| m.is_heading)
            .map(|(k, _)| k.as_str())
            .collect();

        let is_default_heading = |style: &str| -> bool {
            style.starts_with("Heading") || style.starts_with("heading") || style == "Title"
        };

        for para in &paragraphs {
            let is_heading = heading_styles.contains(para.style_id.as_str())
                || is_default_heading(&para.style_id);

            if para.text.trim().is_empty() {
                continue;
            }

            if is_heading {
                // Flush previous heading+body
                flush_heading(
                    &mut pending_heading,
                    &mut body_parts,
                    &mut level_stack,
                    &existing_by_docx_id,
                    &mut objects_created,
                    &mut objects_updated,
                    db,
                    module_id,
                )
                .await?;

                let classification = mapping_by_style
                    .get(&para.style_id)
                    .map_or_else(|| "heading".to_owned(), |m| m.classification.clone());

                pending_heading =
                    Some((para.text.clone(), classification, para.bookmark_id.clone()));
            } else {
                let mapped = mapping_by_style.get(&para.style_id);
                if mapped.is_some() || (pending_heading.is_some() && mapping_by_style.is_empty()) {
                    body_parts.push(para.text.clone());
                } else {
                    paragraphs_skipped += 1;
                }
            }
        }

        // Flush last heading+body
        flush_heading(
            &mut pending_heading,
            &mut body_parts,
            &mut level_stack,
            &existing_by_docx_id,
            &mut objects_created,
            &mut objects_updated,
            db,
            module_id,
        )
        .await?;

        Ok(DocxImportResult {
            objects_created,
            objects_updated,
            paragraphs_skipped,
        })
    }
}

#[allow(clippy::too_many_arguments)]
async fn flush_heading(
    heading: &mut Option<(String, String, Option<String>)>,
    body_parts: &mut Vec<String>,
    level_stack: &mut Vec<(usize, Uuid)>,
    existing_by_docx_id: &std::collections::HashMap<String, object::Model>,
    objects_created: &mut usize,
    objects_updated: &mut usize,
    db: &impl ConnectionTrait,
    module_id: Uuid,
) -> Result<(), CoreError> {
    let h = heading.take();
    let bp = std::mem::take(body_parts);

    if let Some((text, classification, bookmark)) = h {
        let body_text = if bp.is_empty() {
            None
        } else {
            Some(bp.join("\n\n"))
        };

        // Round-trip: check if bookmark maps to existing object
        if let Some(ref bm) = bookmark
            && let Some(existing) = existing_by_docx_id.get(bm)
        {
            let update_input = UpdateObjectInput {
                parent_id: None,
                position: None,
                heading: Some(text),
                body: body_text,
                attributes: None,
                reviewed: None,
                classification: Some(classification),
                references: None,
                object_type_id: None,
                expected_version: None,
                lifecycle_state: None,
            };
            let _ = ObjectService::update(db, existing.id, update_input).await?;
            *objects_updated += 1;
            let depth = existing.level.matches('.').count();
            level_stack.truncate(depth);
            level_stack.push((depth, existing.id));
            return Ok(());
        }

        let depth = level_stack.len();
        let parent_id = level_stack.last().map(|(_, id)| *id);

        let create_input = CreateObjectInput {
            module_id,
            parent_id,
            position: None,
            heading: Some(text),
            body: body_text,
            attributes: None,
            classification: Some(classification),
            references: None,
            object_type_id: None,
            lifecycle_state: None,
            lifecycle_model_id: None,
            source_object_id: None,
            source_module_id: None,
            is_placeholder: None,
        };
        let created = ObjectService::create(db, create_input).await?;

        // Set docx_source_id if bookmark present
        if let Some(bm) = bookmark {
            let mut active: object::ActiveModel = created.clone().into();
            active.docx_source_id = Set(Some(bm));
            let _ = active.update(db).await?;
        }

        level_stack.truncate(depth);
        level_stack.push((depth, created.id));
        *objects_created += 1;
    }
    Ok(())
}

fn parse_docx_paragraphs(data: &[u8]) -> Result<Vec<ParsedParagraph>, CoreError> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| CoreError::bad_request(format!("invalid DOCX file: {e}")))?;

    let mut document_xml = String::new();
    {
        let mut file = archive
            .by_name("word/document.xml")
            .map_err(|e| CoreError::bad_request(format!("missing word/document.xml: {e}")))?;
        let _ = file
            .read_to_string(&mut document_xml)
            .map_err(|e| CoreError::bad_request(format!("cannot read document.xml: {e}")))?;
    }

    parse_document_xml(&document_xml)
}

fn parse_document_xml(xml: &str) -> Result<Vec<ParsedParagraph>, CoreError> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    let mut paragraphs = Vec::new();

    let mut in_paragraph = false;
    let mut in_run = false;
    let mut in_text = false;
    let mut current_style = String::new();
    let mut current_text = String::new();
    let mut current_bookmark: Option<String> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"p" => {
                        in_paragraph = true;
                        current_style.clear();
                        current_text.clear();
                        current_bookmark = None;
                    }
                    b"pStyle" if in_paragraph => {
                        for attr in e.attributes().flatten() {
                            if attr.key.local_name().as_ref() == b"val" {
                                current_style = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    b"r" if in_paragraph => {
                        in_run = true;
                    }
                    b"t" if in_run => {
                        in_text = true;
                    }
                    b"bookmarkStart" if in_paragraph => {
                        for attr in e.attributes().flatten() {
                            if attr.key.local_name().as_ref() == b"name" {
                                let name = String::from_utf8_lossy(&attr.value).to_string();
                                if name.starts_with("req1_") {
                                    current_bookmark = Some(
                                        name.strip_prefix("req1_").unwrap_or(&name).to_string(),
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"p" if in_paragraph => {
                        in_paragraph = false;
                        let style = if current_style.is_empty() {
                            "Normal".to_string()
                        } else {
                            current_style.clone()
                        };
                        paragraphs.push(ParsedParagraph {
                            style_id: style,
                            text: current_text.trim().to_string(),
                            bookmark_id: current_bookmark.take(),
                        });
                    }
                    b"r" => {
                        in_run = false;
                        in_text = false;
                    }
                    b"t" => {
                        in_text = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                if let Ok(text) = e.unescape() {
                    current_text.push_str(&text);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(CoreError::bad_request(format!("XML parse error: {e}")));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(paragraphs)
}
