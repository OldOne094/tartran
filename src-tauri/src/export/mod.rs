use crate::error::{AppError, AppResult};
use crate::models::ExportFile;
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::project_store::ProjectStore;
use base64::Engine;
use std::io::Write;
use std::path::PathBuf;
use zip::write::SimpleFileOptions;

pub struct Exporter<'a> {
    settings: AppSettingsStore,
    logger: &'a crate::logger::Logger,
}

impl<'a> Exporter<'a> {
    pub fn new(settings: AppSettingsStore, logger: &'a crate::logger::Logger) -> Self {
        Exporter { settings, logger }
    }

    fn store_for(&self, project_id: &str) -> AppResult<ProjectStore> {
        let entry = self
            .settings
            .registry()
            .into_iter()
            .find(|e| e.id == project_id)
            .ok_or_else(|| AppError::NotFound(format!("Project not found: {project_id}")))?;
        ProjectStore::open(&PathBuf::from(&entry.path).join("novel.db"))
            .map_err(|e| AppError::Db(e))
    }

    /// Clean copy of a chapter (no metadata/debug).
    pub fn chapter_clean_text(&self, project_id: &str, chapter_id: &str) -> AppResult<ExportFile> {
        let store = self.store_for(project_id)?;
        let row = store
            .get_chapter(chapter_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Chapter not found".into()))?;
        let title = if row.title.trim().is_empty() {
            format!("Chapter {}", row.number)
        } else {
            row.title
        };
        let content = format!("{title}\n\n{}", row.translation);
        self.logger.info(
            "export:chapter_text",
            Some(&serde_json::json!({ "project": project_id, "chapter": chapter_id })),
        );
        Ok(ExportFile {
            name: format!("{}-{}.txt", safe_filename(&title), row.number),
            mime: "text/plain".into(),
            data_base64: base64::engine::general_purpose::STANDARD.encode(content.as_bytes()),
        })
    }

    /// DOCX export of a chapter (translation; RTL paragraph for Arabic target).
    pub fn chapter_docx(&self, project_id: &str, chapter_id: &str, target_lang: &str) -> AppResult<ExportFile> {
        let store = self.store_for(project_id)?;
        let row = store
            .get_chapter(chapter_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Chapter not found".into()))?;
        let title = if row.title.trim().is_empty() {
            format!("Chapter {}", row.number)
        } else {
            row.title
        };
        let paragraphs: Vec<&str> = row.translation.lines().collect();
        let docx = build_docx(&title, &paragraphs, target_lang == "ar")?;
        self.logger.info(
            "export:chapter_docx",
            Some(&serde_json::json!({ "project": project_id, "chapter": chapter_id, "lang": target_lang })),
        );
        Ok(ExportFile {
            name: format!("{}-{}.docx", safe_filename(&title), row.number),
            mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".into(),
            data_base64: base64::engine::general_purpose::STANDARD.encode(&docx),
        })
    }

    /// XLSX export of the glossary: Chinese | English | Arabic | Category | Notes.
    pub fn glossary_xlsx(&self, project_id: &str) -> AppResult<ExportFile> {
        self.logger.info("export:glossary_xlsx", Some(&serde_json::json!({ "project": project_id })));
        let store = self.store_for(project_id)?;
        let rows = store.list_glossary().map_err(|e| AppError::Db(e))?;
        let meta = store.get_meta().map_err(|e| AppError::Db(e))?;
        let title = meta.get("title").cloned().unwrap_or_else(|| "glossary".into());
        let xlsx = build_xlsx(&rows)?;
        Ok(ExportFile {
            name: format!("{}-glossary.xlsx", safe_filename(&title)),
            mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(),
            data_base64: base64::engine::general_purpose::STANDARD.encode(&xlsx),
        })
    }
}

fn safe_filename(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '_' || *c == '-')
        .take(60)
        .collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        "chapter".to_string()
    } else {
        cleaned.replace(' ', "_")
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn build_docx(title: &str, paragraphs: &[&str], rtl: bool) -> AppResult<Vec<u8>> {
    let mut body = String::new();
    body.push_str(&format!(
        "<w:p><w:pPr><w:pStyle w:val=\"Title\"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>",
        escape_xml(title)
    ));
    let rtl_props = if rtl {
        "<w:pPr><w:bidi/></w:pPr>"
    } else {
        ""
    };
    for p in paragraphs {
        if p.trim().is_empty() {
            continue;
        }
        body.push_str(&format!(
            "<w:p>{rtl_props}<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
            escape_xml(p)
        ));
    }

    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    {body}
  </w:body>
</w:document>"#
    );
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:style>
</w:styles>"#;
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

    let cursor = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(cursor);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    add_zip_entry(&mut zip, &opts, "[Content_Types].xml", content_types)?;
    add_zip_entry(&mut zip, &opts, "_rels/.rels", rels)?;
    add_zip_entry(&mut zip, &opts, "word/document.xml", &document)?;
    add_zip_entry(&mut zip, &opts, "word/styles.xml", styles)?;
    add_zip_entry(&mut zip, &opts, "word/_rels/document.xml.rels", doc_rels)?;
    let inner = zip.finish().map_err(|e| AppError::Export(e.to_string()))?;
    Ok(inner.into_inner())
}

fn build_xlsx(rows: &[crate::storage::project_store::GlossaryRow]) -> AppResult<Vec<u8>> {
    let mut sheet = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Chinese</t></is></c>
      <c r="B1" t="inlineStr"><is><t>English</t></is></c>
      <c r="C1" t="inlineStr"><is><t>Arabic</t></is></c>
      <c r="D1" t="inlineStr"><is><t>Category</t></is></c>
      <c r="E1" t="inlineStr"><is><t>Notes</t></is></c>
    </row>
"#,
    );
    for (i, row) in rows.iter().enumerate() {
        let r = i + 2;
        let cells = [
            format!("A{r}"),
            format!("B{r}"),
            format!("C{r}"),
            format!("D{r}"),
            format!("E{r}"),
        ];
        let values = [
            row.zh.clone(),
            row.en.clone(),
            row.ar.clone(),
            row.category.clone(),
            row.notes.clone(),
        ];
        for (col, val) in cells.iter().zip(values.iter()) {
            sheet.push_str(&format!(
                "<c r=\"{col}\" t=\"inlineStr\"><is><t>{}</t></is></c>",
                escape_xml(val)
            ));
        }
        sheet.push_str("</row>\n");
    }
    sheet.push_str("</sheetData></worksheet>");

    let workbook = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Glossary" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#;
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;
    let workbook_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#;

    let cursor = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(cursor);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    add_zip_entry(&mut zip, &opts, "[Content_Types].xml", content_types)?;
    add_zip_entry(&mut zip, &opts, "_rels/.rels", rels)?;
    add_zip_entry(&mut zip, &opts, "xl/workbook.xml", workbook)?;
    add_zip_entry(&mut zip, &opts, "xl/_rels/workbook.xml.rels", workbook_rels)?;
    add_zip_entry(&mut zip, &opts, "xl/worksheets/sheet1.xml", &sheet)?;
    let inner = zip.finish().map_err(|e| AppError::Export(e.to_string()))?;
    Ok(inner.into_inner())
}

fn add_zip_entry(
    zip: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>,
    opts: &SimpleFileOptions,
    name: &str,
    content: &str,
) -> AppResult<()> {
    zip.start_file(name, *opts)
        .map_err(|e| AppError::Export(e.to_string()))?;
    zip.write_all(content.as_bytes())
        .map_err(|e| AppError::Export(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_filenames() {
        assert_eq!(safe_filename("The Lord of the Rings"), "The_Lord_of_the_Rings");
        assert_eq!(safe_filename(""), "chapter");
    }

    #[test]
    fn builds_valid_docx_container() {
        let bytes = build_docx("Title", &["Para one", "Para two"], true).unwrap();
        let reader = std::io::Cursor::new(bytes);
        let mut zip = zip::ZipArchive::new(reader).unwrap();
        assert!(zip.file_names().any(|n| n == "word/document.xml"));
    }

    #[test]
    fn builds_valid_xlsx_container() {
        let rows = vec![crate::storage::project_store::GlossaryRow {
            id: "1".into(),
            zh: "剑客".into(),
            en: "swordsman".into(),
            ar: "سياف".into(),
            category: "character".into(),
            notes: String::new(),
            aliases: "[]".into(),
            locked: false,
            source: "manual".into(),
            created_at: String::new(),
            updated_at: String::new(),
        }];
        let bytes = build_xlsx(&rows).unwrap();
        let reader = std::io::Cursor::new(bytes);
        let mut zip = zip::ZipArchive::new(reader).unwrap();
        assert!(zip.file_names().any(|n| n == "xl/worksheets/sheet1.xml"));
    }
}
