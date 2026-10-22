use crate::{files::SaveMode, metadata::AppResult};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

static WRITES: Mutex<()> = Mutex::new(());

pub fn update(
    path: &Path,
    documents: &Path,
    change: impl FnOnce(&mut Preferences),
) -> AppResult<Preferences> {
    let _guard = WRITES.lock().map_err(|error| error.to_string())?;
    let mut value = load(path, documents)?;
    change(&mut value);
    save(path, &value)?;
    Ok(value)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub theme: Theme,
    pub save_mode: SaveMode,
    pub output_dir: PathBuf,
    #[serde(default)]
    pub folders: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Id,
    En,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

pub fn load(path: &Path, documents: &Path) -> AppResult<Preferences> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // Windows also reports NotFound when an ancestor is a file.
            if path.ancestors().skip(1).any(|parent| parent.is_file()) {
                return Err("Lokasi pengaturan tidak valid.".into());
            }
            return Ok(Preferences {
                language: Language::Id,
                theme: Theme::Light,
                save_mode: SaveMode::Copy,
                output_dir: documents.join("stock-assets"),
                folders: Vec::new(),
            });
        }
        Err(error) => return Err(format!("Pengaturan aplikasi tidak dapat dibaca: {error}")),
    };
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("Pengaturan aplikasi tidak valid: {error}"))
}

pub fn save(path: &Path, value: &Preferences) -> AppResult<()> {
    if !value.output_dir.is_absolute() {
        return Err("Direktori keluaran harus berupa path absolut.".into());
    }
    if value.output_dir.exists() && !value.output_dir.is_dir() {
        return Err("Tujuan keluaran harus berupa direktori.".into());
    }
    let parent = path.parent().ok_or("Lokasi pengaturan tidak valid.")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    temporary
        .write_all(&bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    temporary.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_folder_updates_preserve_saved_settings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let barrier = std::sync::Barrier::new(8);
        std::thread::scope(|scope| {
            for index in 0..8 {
                let path = &path;
                let documents = directory.path();
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    update(path, documents, |value| {
                        if index == 0 {
                            value.language = Language::En;
                            value.theme = Theme::Dark;
                            value.save_mode = SaveMode::ReplaceWithBackup;
                            value.output_dir = documents.join("custom-output");
                        } else {
                            value.folders.push(format!("/assets/{index}"));
                        }
                    })
                    .unwrap();
                });
            }
        });
        let restored = load(&path, directory.path()).unwrap();
        assert_eq!(restored.folders.len(), 7);
        assert_eq!(restored.language, Language::En);
        assert_eq!(restored.theme, Theme::Dark);
        assert!(matches!(restored.save_mode, SaveMode::ReplaceWithBackup));
        assert_eq!(restored.output_dir, directory.path().join("custom-output"));
        update(&path, directory.path(), |value| value.folders.clear()).unwrap();
        let restored = load(&path, directory.path()).unwrap();
        assert!(restored.folders.is_empty());
        assert_eq!(restored.language, Language::En);
        assert_eq!(restored.theme, Theme::Dark);
        assert_eq!(restored.output_dir, directory.path().join("custom-output"));
    }

    #[test]
    fn failed_update_preserves_existing_configuration() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        update(&path, directory.path(), |value| value.theme = Theme::Dark).unwrap();
        let saved = fs::read(&path).unwrap();
        assert!(update(&path, directory.path(), |value| value.output_dir = "relative".into()).is_err());
        assert_eq!(fs::read(&path).unwrap(), saved);
        fs::write(&path, b"invalid").unwrap();
        assert!(update(&path, directory.path(), |value| value.folders.clear()).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"invalid");
        // An inaccessible path must not be treated as a missing configuration.
        assert!(load(&path.join("preferences.json"), directory.path()).is_err());
    }

    #[test]
    fn preferences_default_roundtrip_and_invalid_data() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config/preferences.json");
        let mut preferences = load(&path, directory.path()).unwrap();
        assert_eq!(
            preferences.output_dir,
            directory.path().join("stock-assets")
        );
        assert!(matches!(preferences.save_mode, SaveMode::Copy));
        preferences.language = Language::En;
        preferences.theme = Theme::Dark;
        preferences.save_mode = SaveMode::ReplaceWithBackup;
        preferences.folders.push("/missing/assets".into());
        save(&path, &preferences).unwrap();
        let restored = load(&path, directory.path()).unwrap();
        assert!(matches!(restored.save_mode, SaveMode::ReplaceWithBackup));
        assert_eq!(restored.folders, preferences.folders);
        assert_eq!(restored.language, Language::En);
        assert_eq!(restored.theme, Theme::Dark);
        preferences.output_dir = "relative".into();
        assert!(save(&path, &preferences).is_err());
        assert_eq!(
            load(&path, directory.path()).unwrap().output_dir,
            restored.output_dir
        );
        fs::write(path, b"invalid").unwrap();
        assert!(load(
            &directory.path().join("config/preferences.json"),
            directory.path()
        )
        .is_err());
    }
    #[test]
    fn legacy_preferences_default_to_indonesian_and_light() {
        let legacy = r#"{"saveMode":"copy","outputDir":"/tmp/output","folders":["/tmp/assets"]}"#;
        let restored: Preferences = serde_json::from_str(legacy).unwrap();
        assert_eq!(restored.language, Language::Id);
        assert_eq!(restored.theme, Theme::Light);
        assert_eq!(restored.folders, ["/tmp/assets"]);
        assert!(serde_json::from_str::<Preferences>(
            &legacy.replace("\"copy\"", "\"copy\",\"language\":\"fr\"")
        )
        .is_err());
    }
}
