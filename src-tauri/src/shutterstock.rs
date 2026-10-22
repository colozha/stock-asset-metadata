//! Shutterstock's video CSV export, independent of embedding and source files.
use crate::metadata::{AppResult, Metadata};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize)]
pub struct CsvItem {
    pub filename: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvResult {
    pub directory: PathBuf,
    pub output_path: Option<PathBuf>,
    pub row_count: usize,
    #[serde(serialize_with = "crate::messages::serialize_error")]
    pub error: Option<String>,
}

pub fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("mp4") || value.eq_ignore_ascii_case("mov"))
}

// Validate before any asset is written. Validate names individually because copies
// can acquire collision suffixes; the final batch validates the actual output names.
pub fn validate_save(category: &str, items: &[crate::files::SaveItem]) -> AppResult<()> {
    for item in items.iter().filter(|item| is_video(Path::new(&item.path))) {
        let path = Path::new(&item.path);
        let metadata = item.patch.apply(&crate::metadata::read(path)?)?;
        serialize(
            category,
            &[CsvItem {
                filename: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                metadata,
            }],
        )?;
    }
    Ok(())
}

pub fn export_saved(category: &str, results: &[crate::files::SaveResult]) -> Vec<CsvResult> {
    let mut groups: BTreeMap<PathBuf, Vec<CsvItem>> = BTreeMap::new();
    for result in results.iter().filter(|result| result.error.is_none()) {
        if let (Some(output), Some(metadata)) = (&result.output_path, &result.metadata) {
            let path = Path::new(output);
            if is_video(path) {
                if let (Some(parent), Some(name)) = (path.parent(), path.file_name()) {
                    groups.entry(parent.to_owned()).or_default().push(CsvItem {
                        filename: name.to_string_lossy().into_owned(),
                        metadata: metadata.clone(),
                    });
                }
            }
        }
    }
    groups
        .into_iter()
        .map(|(directory, items)| {
            let outcome = export(&directory, category, &items);
            let (output_path, error) = match outcome {
                Ok(path) => (Some(path), None),
                Err(error) => (None, Some(error)),
            };
            CsvResult {
                directory,
                output_path,
                row_count: items.len(),
                error,
            }
        })
        .collect()
}

fn csv_cell(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub fn serialize(category: &str, items: &[CsvItem]) -> AppResult<String> {
    let categories: Vec<String> =
        serde_json::from_str(include_str!("../../src/shutterstock-categories.json"))
            .map_err(|e| e.to_string())?;
    if !categories.iter().any(|value| value == category) {
        return Err("Pilih kategori video Shutterstock yang valid.".into());
    }
    if items.is_empty() {
        return Err("Pilih video MP4/MOV untuk diekspor.".into());
    }
    let mut names = HashSet::new();
    let mut csv = "\"Filename\",\"Description\",\"Keywords\",\"Categories\"\r\n".to_owned();
    for item in items {
        let name = &item.filename;
        if name.contains(['/', '\\'])
            || name.chars().any(char::is_control)
            || !["mp4", "mov"].contains(
                &std::path::Path::new(name)
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .as_str(),
            )
        {
            return Err(format!("Nama video CSV tidak valid: {name}"));
        }
        if !names.insert(name.to_lowercase()) {
            return Err(format!(
                "Nama file CSV duplikat: {name}. Ekspor batch terpisah atau ubah nama file."
            ));
        }
        let description = &item.metadata.description;
        if description.split_whitespace().count() < 5 || description.chars().count() > 2048 {
            return Err(format!(
                "Deskripsi {name} harus berisi minimal 5 kata dan maksimal 2048 karakter."
            ));
        }
        let keywords = &item.metadata.keywords;
        let unique: HashSet<_> = keywords
            .iter()
            .map(|word| word.trim().to_lowercase())
            .collect();
        if !(7..=50).contains(&keywords.len())
            || unique.len() != keywords.len()
            || keywords
                .iter()
                .any(|word| word.trim().is_empty() || word.contains(','))
        {
            return Err(format!(
                "Keyword {name} harus berisi 7–50 frasa unik, tanpa frasa kosong atau koma."
            ));
        }
        if std::iter::once(description)
            .chain(keywords.iter())
            .any(|value| {
                value
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
            })
        {
            return Err(format!(
                "Metadata CSV mengandung karakter kontrol yang tidak valid: {name}"
            ));
        }
        let joined = keywords.join(", ");
        csv.push_str(
            &[name.as_str(), description, &joined, category]
                .map(csv_cell)
                .join(","),
        );
        csv.push_str("\r\n");
    }
    Ok(csv)
}

pub fn export(directory: &Path, category: &str, items: &[CsvItem]) -> AppResult<PathBuf> {
    let csv = serialize(category, items)?;
    if !directory.is_absolute() {
        return Err("Direktori keluaran harus berupa path absolut.".into());
    }
    let mut temporary = tempfile::Builder::new()
        .prefix(".shutterstock-csv-")
        .tempfile_in(directory)
        .map_err(|e| e.to_string())?;
    temporary
        .write_all(csv.as_bytes())
        .map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    let mut output = directory.join("shutterstock.csv");
    let mut index = 1;
    loop {
        match temporary.persist_noclobber(&output) {
            Ok(_) => break,
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                temporary = error.file;
                output = directory.join(format!("shutterstock_{index}.csv"));
                index += 1;
            }
            Err(error) => return Err(error.error.to_string()),
        }
    }
    #[cfg(unix)]
    fs::File::open(directory)
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("Hasil sudah ditulis, tetapi sinkronisasi folder gagal: {e}"))?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item() -> CsvItem {
        CsvItem {
            filename: "clip_1.MP4".into(),
            metadata: Metadata {
                title: "Do not use the title".into(),
                description: "Colorful, \"glowing\" particles drift 日本.\nSecond line.".into(),
                keywords: [
                    "motion graphics",
                    "particles",
                    "日本",
                    "café",
                    "glow",
                    "background",
                    "blue",
                ]
                .map(str::to_owned)
                .to_vec(),
            },
        }
    }

    #[test]
    fn csv_quotes_preserves_order_and_validates_whole_batch() {
        let valid = item();
        let csv = serialize("Backgrounds/Textures", &[valid.clone()]).unwrap();
        assert!(csv.starts_with("\"Filename\",\"Description\",\"Keywords\",\"Categories\"\r\n"));
        assert!(csv.contains("\"Colorful, \"\"glowing\"\" particles drift 日本.\nSecond line.\""));
        assert!(csv.contains("\"motion graphics, particles, 日本, café, glow, background, blue\""));
        assert!(csv.ends_with("\"Backgrounds/Textures\"\r\n"));
        assert!(!csv.contains("Do not use the title"));
        assert!(serialize("Abstract", &[valid.clone()]).is_err());
        assert!(serialize("Nature", &[]).is_err());
        let mut bad = valid.clone();
        bad.filename = "CLIP_1.mp4".into();
        assert!(serialize("Nature", &[valid.clone(), bad]).is_err());
        for invalid in [
            "picture.jpg",
            "../clip.mp4",
            "folder\\clip.mov",
            "bad\nname.mp4",
        ] {
            let mut bad = valid.clone();
            bad.filename = invalid.into();
            assert!(serialize("Nature", &[bad]).is_err());
        }
        for invalid in ["short", "", &"a ".repeat(1025)] {
            let mut bad = valid.clone();
            bad.metadata.description = invalid.into();
            assert!(serialize("Nature", &[valid.clone(), bad]).is_err());
        }
        for words in [
            vec!["x"; 6],
            vec!["x"; 51],
            vec![""; 7],
            vec!["a,b"; 7],
            vec!["X", "x", "a", "b", "c", "d", "e"],
        ] {
            let mut bad = valid.clone();
            bad.metadata.keywords = words.into_iter().map(str::to_owned).collect();
            assert!(serialize("Nature", &[bad]).is_err());
        }
        let mut mov = valid.clone();
        mov.filename = "clip.mov".into();
        let mut boundary = valid.clone();
        boundary.metadata.description = format!("{}abc", "word ".repeat(409));
        assert_eq!(boundary.metadata.description.chars().count(), 2048);
        boundary.metadata.keywords = (0..50).map(|n| format!("keyword {n}")).collect();
        assert!(serialize("Nature", &[boundary.clone()]).is_ok());
        boundary.metadata.keywords.push("keyword 50".into());
        assert!(serialize("Nature", &[boundary.clone()]).is_err());
        boundary.metadata.keywords.truncate(6);
        assert!(serialize("Nature", &[boundary]).is_err());
        let mut control = valid.clone();
        control.metadata.keywords[0].push('\0');
        assert!(serialize("Nature", &[control]).is_err());
        assert!(serialize("Nature", &[valid, mov]).is_ok());
    }

    #[test]
    fn export_is_atomic_and_preserves_existing_csv() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("shutterstock.csv");
        fs::write(&output, "existing CSV").unwrap();
        let mut invalid = item();
        invalid.metadata.keywords.clear();
        assert!(export(directory.path(), "Nature", &[item(), invalid]).is_err());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
        let result = export(directory.path(), "Nature", &[item()]).unwrap();
        assert_eq!(result, directory.path().join("shutterstock_1.csv"));
        assert_eq!(
            fs::read_to_string(result).unwrap(),
            serialize("Nature", &[item()]).unwrap()
        );
        assert_eq!(fs::read_to_string(output).unwrap(), "existing CSV");
        assert!(export(&directory.path().join("missing"), "Nature", &[item()]).is_err());
        assert!(export(Path::new("relative"), "Nature", &[item()]).is_err());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }
    #[test]
    fn saved_outputs_group_by_folder_skip_failures_and_continue_after_csv_errors() {
        use crate::files::SaveResult;
        let directory = tempfile::tempdir().unwrap();
        let mp4 = directory.path().join("mp4");
        let mov = directory.path().join("mov");
        fs::create_dir(&mp4).unwrap();
        fs::create_dir(&mov).unwrap();
        let saved = |path: PathBuf| SaveResult {
            path: "source.mp4".into(),
            output_path: Some(path.to_string_lossy().into_owned()),
            metadata: Some(item().metadata),
            backup_path: None,
            error: None,
        };
        let mut failed = saved(mp4.join("failed.mp4"));
        failed.error = Some("write failed".into());
        let results = vec![
            saved(mp4.join("clip_2.mp4")),
            saved(mp4.join("other.MP4")),
            saved(mov.join("clip.mov")),
            saved(mp4.join("image.png")),
            failed,
            saved(directory.path().join("a-missing/clip.mp4")),
        ];
        let exports = export_saved("Nature", &results);
        assert_eq!(exports.len(), 3);
        assert!(exports[0].error.is_some());
        assert!(exports[0].output_path.is_none());
        for result in &exports[1..] {
            assert!(result.error.is_none());
        }
        let csv = fs::read_to_string(mp4.join("shutterstock.csv")).unwrap();
        assert!(csv.contains("\"clip_2.mp4\""));
        assert!(csv.contains("\"other.MP4\""));
        assert!(!csv.contains("failed.mp4") && !csv.contains("image.png"));
        assert_eq!(
            exports.iter().map(|result| result.row_count).sum::<usize>(),
            4
        );
        assert!(fs::read_to_string(mov.join("shutterstock.csv"))
            .unwrap()
            .contains("clip.mov"));
        assert!(export_saved("Nature", &[]).is_empty());
    }

    #[test]
    fn preflight_validates_patched_metadata_without_writing_sources() {
        use crate::{files::SaveItem, metadata::MetadataPatch};
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("clip.mp4");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/clip.mp4");
        fs::copy(fixture, &source).unwrap();
        let original = fs::read(&source).unwrap();
        let metadata = item().metadata;
        let mut item = SaveItem {
            path: source.to_string_lossy().into_owned(),
            patch: MetadataPatch {
                title: None,
                description: Some(metadata.description),
                keywords: Some(metadata.keywords),
            },
        };
        validate_save("Nature", &[item.clone()]).unwrap();
        assert!(validate_save("invalid", &[item.clone()]).is_err());
        item.patch.description = Some("too short".into());
        assert!(validate_save("Nature", &[item]).is_err());
        assert_eq!(fs::read(source).unwrap(), original);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
