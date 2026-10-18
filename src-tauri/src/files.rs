use crate::metadata::{self, AppResult, Metadata, MetadataPatch};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStamp {
    len: u64,
    modified: SystemTime,
}

pub fn stamp(path: &Path) -> AppResult<FileStamp> {
    let stat = fs::metadata(path).map_err(|e| e.to_string())?;
    if !stat.is_file() {
        return Err("Sumber bukan file biasa.".into());
    }
    Ok(FileStamp {
        len: stat.len(),
        modified: stat.modified().map_err(|e| e.to_string())?,
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub path: String,
    pub name: String,
    pub format: String,
    pub size: u64,
    pub metadata: Metadata,
    pub error: Option<String>,
}

pub fn inspect(paths: Vec<String>) -> (Vec<Asset>, HashMap<PathBuf, FileStamp>) {
    let mut candidates = BTreeMap::new();
    for name in paths {
        let path = PathBuf::from(name);
        let children = if path.is_dir() {
            match fs::read_dir(&path) {
                Ok(items) => items.filter_map(Result::ok).map(|i| i.path()).collect(),
                Err(_) => vec![path],
            }
        } else {
            vec![path]
        };
        for path in children {
            if path.is_dir()
                || fs::symlink_metadata(&path)
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false)
            {
                continue;
            }
            if metadata::format(&path).is_err() {
                continue;
            }
            let canonical = path.canonicalize().unwrap_or(path);
            candidates.insert(canonical.clone(), canonical);
        }
    }
    let mut stamps = HashMap::new();
    let assets = candidates
        .into_values()
        .map(|path| {
            let snapshot = stamp(&path);
            let size = snapshot.as_ref().map(|s| s.len).unwrap_or(0);
            let result = snapshot.and_then(|s| {
                let value = metadata::read(&path)?;
                if stamp(&path)? != s {
                    return Err("File berubah saat dibaca. Impor ulang file.".into());
                }
                stamps.insert(path.clone(), s);
                Ok(value)
            });
            Asset {
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                format: metadata::format(&path).unwrap_or_default(),
                size,
                path: path.to_string_lossy().into_owned(),
                error: result.as_ref().err().cloned(),
                metadata: result.unwrap_or_default(),
            }
        })
        .collect();
    (assets, stamps)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveMode {
    Copy,
    ReplaceWithBackup,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveItem {
    pub path: String,
    pub patch: MetadataPatch,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRequest {
    pub items: Vec<SaveItem>,
    pub mode: SaveMode,
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub path: String,
    pub output_path: Option<String>,
    pub backup_path: Option<String>,
    pub metadata: Option<Metadata>,
    pub error: Option<String>,
}

pub fn save_one(
    item: &SaveItem,
    mode: &SaveMode,
    output_dir: Option<&Path>,
    expected: &FileStamp,
) -> AppResult<SaveResult> {
    let source = Path::new(&item.path);
    if stamp(source)? != *expected {
        return Err("Sumber berubah sejak diimpor. Impor ulang sebelum menyimpan.".into());
    }
    let value = item.patch.apply(&metadata::read(source)?)?;
    let parent = match mode {
        SaveMode::Copy => output_dir.ok_or("Pilih folder keluaran terlebih dahulu.")?,
        SaveMode::ReplaceWithBackup => source.parent().ok_or("Folder sumber tidak tersedia.")?,
    };
    if !parent.is_dir() {
        return Err("Folder keluaran tidak tersedia.".into());
    }
    let extension = source.extension().and_then(|s| s.to_str()).unwrap_or("");
    let temporary = tempfile::Builder::new()
        .prefix(".stock-metadata-")
        .suffix(&format!(".{extension}"))
        .tempfile_in(parent)
        .map_err(|e| e.to_string())?;
    fs::copy(source, temporary.path()).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(temporary.path())
            .map_err(|e| e.to_string())?
            .permissions()
            .mode();
        fs::set_permissions(temporary.path(), fs::Permissions::from_mode(mode | 0o200))
            .map_err(|e| e.to_string())?;
    }
    let output_permissions = fs::metadata(temporary.path())
        .map_err(|e| e.to_string())?
        .permissions();
    metadata::write(temporary.path(), &value)?;
    // The SDK can replace the inode, so restore the intended source permissions.
    fs::set_permissions(temporary.path(), output_permissions).map_err(|e| e.to_string())?;
    if metadata::read(temporary.path())? != value {
        return Err("Pembacaan ulang metadata tidak sesuai. Sumber tidak diganti.".into());
    }
    // A format handler may replace the temporary inode while updating metadata.
    fs::File::open(temporary.path())
        .and_then(|file| file.sync_all())
        .map_err(|e| e.to_string())?;
    if stamp(source)? != *expected {
        return Err("Sumber berubah selama penyimpanan. Hasil dibatalkan.".into());
    }
    let mut backup_path = None;
    let output = match mode {
        SaveMode::Copy => {
            let stem = source.file_stem().unwrap_or_default().to_string_lossy();
            let name = source.file_name().ok_or("Nama file tidak valid.")?;
            let mut destination = parent.join(name);
            let mut index = 1;
            let mut temp = temporary;
            loop {
                match temp.persist_noclobber(&destination) {
                    Ok(_) => break destination,
                    Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                        temp = error.file;
                        destination = parent.join(format!("{stem}_{index}.{extension}"));
                        index += 1;
                    }
                    Err(error) => return Err(error.error.to_string()),
                }
            }
        }
        SaveMode::ReplaceWithBackup => {
            let permissions = fs::metadata(source)
                .map_err(|e| e.to_string())?
                .permissions();
            if permissions.readonly() {
                return Err("Sumber bersifat read-only. Gunakan Simpan salinan.".into());
            }
            let base = format!(
                "{}.bak",
                source.file_name().unwrap_or_default().to_string_lossy()
            );
            let mut destination = parent.join(&base);
            let mut index = 1;
            loop {
                match fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&destination)
                {
                    Ok(mut backup) => {
                        let result = (|| {
                            backup.set_permissions(permissions.clone())?;
                            std::io::copy(&mut fs::File::open(source)?, &mut backup)?;
                            backup.sync_all()?;
                            Ok::<_, std::io::Error>(())
                        })();
                        if let Err(error) = result {
                            let _ = fs::remove_file(&destination);
                            return Err(error.to_string());
                        }
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        destination = parent.join(format!("{base}.{index}"));
                        index += 1;
                    }
                    Err(error) => return Err(error.to_string()),
                }
            }
            if stamp(source)? != *expected {
                return Err("Sumber berubah saat membuat backup. Sumber tidak diganti.".into());
            }
            temporary.persist(source).map_err(|e| e.error.to_string())?;
            backup_path = Some(destination.to_string_lossy().into_owned());
            source.to_owned()
        }
    };
    // Persist the directory entry as well as the new file contents on Linux.
    fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| format!("Hasil sudah ditulis, tetapi sinkronisasi folder gagal: {e}"))?;
    Ok(SaveResult {
        path: item.path.clone(),
        output_path: Some(output.to_string_lossy().into_owned()),
        backup_path,
        metadata: Some(value),
        error: None,
    })
}
