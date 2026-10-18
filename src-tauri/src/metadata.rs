use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};
use xmp_toolkit::{OpenFileOptions, XmpFile, XmpMeta, XmpValue};

pub const DC: &str = "http://purl.org/dc/elements/1.1/";
pub type AppResult<T> = Result<T, String>;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
}

impl MetadataPatch {
    pub fn apply(&self, original: &Metadata) -> AppResult<Metadata> {
        let mut seen = HashSet::new();
        let metadata = Metadata {
            title: self
                .title
                .as_ref()
                .unwrap_or(&original.title)
                .trim()
                .to_owned(),
            description: self
                .description
                .as_ref()
                .unwrap_or(&original.description)
                .trim()
                .to_owned(),
            keywords: self
                .keywords
                .as_ref()
                .unwrap_or(&original.keywords)
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .filter(|s| seen.insert(s.to_lowercase()))
                .map(str::to_owned)
                .collect(),
        };
        for value in [&metadata.title, &metadata.description]
            .into_iter()
            .chain(metadata.keywords.iter())
        {
            if value.chars().any(|c| {
                (c < ' ' && !matches!(c, '\t' | '\n' | '\r'))
                    || matches!(c, '\u{fffe}' | '\u{ffff}')
            }) {
                return Err("Metadata mengandung karakter yang tidak valid untuk XML.".into());
            }
        }
        Ok(metadata)
    }
}

pub fn format(path: &Path) -> AppResult<String> {
    let extension = path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Ok("jpeg".into()),
        "png" | "svg" | "eps" | "mp4" | "mov" => Ok(extension),
        _ => Err("Format tidak didukung. Gunakan JPG, PNG, SVG, EPS, MP4, atau MOV.".into()),
    }
}

pub fn fields(xmp: &XmpMeta) -> Metadata {
    let localized = |name: &str| {
        xmp.localized_text(DC, name, None, "x-default")
            .map(|(v, _)| v.value)
            .unwrap_or_default()
    };
    let mut keywords = Vec::new();
    for index in 1.. {
        match xmp.array_item(DC, "subject", index) {
            Some(value) => keywords.push(value.value),
            None => break,
        }
    }
    Metadata {
        title: localized("title"),
        description: localized("description"),
        keywords,
    }
}

pub fn update(xmp: &mut XmpMeta, metadata: &Metadata) -> AppResult<()> {
    for (name, value) in [
        ("title", &metadata.title),
        ("description", &metadata.description),
    ] {
        if value.is_empty() {
            xmp.delete_property(DC, name).map_err(|e| e.to_string())?;
        } else {
            xmp.set_localized_text(DC, name, None, "x-default", value)
                .map_err(|e| e.to_string())?;
        }
    }
    xmp.delete_property(DC, "subject")
        .map_err(|e| e.to_string())?;
    for keyword in &metadata.keywords {
        xmp.append_array_item(
            DC,
            &XmpValue::new("subject".into()).set_is_array(true),
            &XmpValue::new(keyword.clone()),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn read(path: &Path) -> AppResult<Metadata> {
    if format(path)? == "svg" {
        return crate::svg::read(path);
    }
    let mut file = XmpFile::new().map_err(|e| e.to_string())?;
    file.open_file(
        path,
        OpenFileOptions::default().for_read().use_smart_handler(),
    )
    .map_err(|e| e.to_string())?;
    let result = file.xmp().map(|xmp| fields(&xmp)).unwrap_or_default();
    file.try_close().map_err(|e| e.to_string())?;
    Ok(result)
}

pub fn write(path: &Path, metadata: &Metadata) -> AppResult<()> {
    if format(path)? == "svg" {
        return crate::svg::write(path, metadata);
    }
    let mut file = XmpFile::new().map_err(|e| e.to_string())?;
    file.open_file(
        path,
        OpenFileOptions::default().for_update().use_smart_handler(),
    )
    .map_err(|e| e.to_string())?;
    let mut xmp = file
        .xmp()
        .unwrap_or(XmpMeta::new().map_err(|e| e.to_string())?);
    update(&mut xmp, metadata)?;
    if !file.can_put_xmp(&xmp) {
        return Err("Handler tidak dapat menulis metadata pada file ini.".into());
    }
    file.put_xmp(&xmp).map_err(|e| e.to_string())?;
    file.try_close().map_err(|e| e.to_string())
}
