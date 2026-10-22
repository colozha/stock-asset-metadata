pub mod files;
pub mod messages;
pub mod metadata;
pub mod preferences;
pub mod preview;
pub mod shutterstock;
pub mod svg;

#[cfg(feature = "desktop")]
mod ai;

#[cfg(feature = "desktop")]
mod desktop {
    use super::{
        ai::{self, AiGenerationRequest, AiSettingsInput, ProviderDraft},
        files::{self, FileStamp, SaveRequest, SaveResponse, SaveResult},
        preferences::{self, Preferences},
        preview,
    };
    use std::{
        collections::{HashMap, HashSet},
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };
    use tauri::{Emitter, Manager, State};

    #[derive(Default)]
    struct AppState {
        snapshots: Arc<Mutex<HashMap<PathBuf, FileStamp>>>,
        output_previews: Mutex<HashSet<PathBuf>>,
        busy: Mutex<bool>,
        previews: Arc<Mutex<()>>,
    }

    struct BusyGuard<'a>(&'a Mutex<bool>);
    impl Drop for BusyGuard<'_> {
        fn drop(&mut self) {
            if let Ok(mut value) = self.0.lock() {
                *value = false;
            }
        }
    }

    fn lock_worker(state: &AppState) -> Result<BusyGuard<'_>, String> {
        let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
        if *busy {
            return Err("Pemrosesan lain masih berlangsung. Tunggu hingga selesai.".into());
        }
        *busy = true;
        Ok(BusyGuard(&state.busy))
    }

    #[tauri::command]
    async fn inspect_assets(
        paths: Vec<String>,
        refresh: Option<bool>,
        preserve_paths: Option<Vec<String>>,
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<files::Inspection, super::messages::AppError> {
        let _worker = lock_worker(&state)?;
        let known = load_app_preferences(app.clone())?.folders;
        let (report, stamps) = tauri::async_runtime::spawn_blocking(move || {
            files::inspect_with_folders(paths, &known)
        })
        .await
        .map_err(|e| e.to_string())?;
        let mut snapshots = state.snapshots.lock().map_err(|e| e.to_string())?;
        for (path, stamp) in stamps {
            if refresh.unwrap_or(false)
                && !preserve_paths.as_ref().is_some_and(|paths| {
                    paths.iter().any(|saved| Path::new(saved) == path)
                })
            {
                snapshots.insert(path, stamp);
            } else {
                snapshots.entry(path).or_insert(stamp);
            }
        }
        drop(snapshots);
        if !refresh.unwrap_or(false) {
            update_app_preferences(&app, |preferences| {
                for folder in &report.folders {
                    if !preferences.folders.contains(&folder.path) {
                        preferences.folders.push(folder.path.clone());
                    }
                }
            })?;
        }
        Ok(report)
    }

    #[tauri::command]
    async fn inspect_output_assets(
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<files::Inspection, super::messages::AppError> {
        let _worker = lock_worker(&state)?;
        let root = load_app_preferences(app)?
            .output_dir
            .to_string_lossy()
            .into_owned();
        let (report, stamps) = tauri::async_runtime::spawn_blocking(move || {
            files::inspect_with_folders(vec![root.clone()], &[root])
        })
        .await
        .map_err(|error| error.to_string())?;
        // Output previews never renew conflict snapshots for editable sources.
        *state.output_previews.lock().map_err(|error| error.to_string())? =
            stamps.into_keys().collect();
        Ok(report)
    }

    fn preferences_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        Ok(app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join("app-preferences.json"))
    }

    fn documents_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        app.path()
            .document_dir()
            .or_else(|_| app.path().home_dir().map(|home| home.join("Documents")))
            .map_err(|error| format!("Direktori Documents tidak tersedia: {error}"))
    }

    fn update_app_preferences(
        app: &tauri::AppHandle,
        change: impl FnOnce(&mut Preferences),
    ) -> Result<Preferences, super::messages::AppError> {
        preferences::update(&preferences_path(app)?, &documents_directory(app)?, change)
            .map_err(Into::into)
    }

    fn preview_cache_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        Ok(app
            .path()
            .app_cache_dir()
            .map_err(|e| e.to_string())?
            .join("previews"))
    }

    #[tauri::command]
    fn load_app_preferences(
        app: tauri::AppHandle,
    ) -> Result<Preferences, super::messages::AppError> {
        let documents = documents_directory(&app)?;
        preferences::load(&preferences_path(&app)?, &documents).map_err(Into::into)
    }

    #[tauri::command]
    async fn open_output_directory(
        app: tauri::AppHandle,
    ) -> Result<(), super::messages::AppError> {
        let directory = load_app_preferences(app)?.output_dir;
        tauri::async_runtime::spawn_blocking(move || {
            if !directory.is_dir() {
                return Err("Folder keluaran tidak tersedia.".to_owned());
            }
            let result = std::process::Command::new("xdg-open")
                .arg(&directory)
                .output()
                .map_err(|e| format!("Direktori output tidak dapat dibuka: {e}"))?;
            if !result.status.success() {
                return Err(format!(
                    "Direktori output tidak dapat dibuka: {}",
                    String::from_utf8_lossy(&result.stderr).trim()
                ));
            }
            Ok(())
        })
        .await
        .map_err(|e| format!("Direktori output tidak dapat dibuka: {e}"))?
        .map_err(Into::into)
    }

    #[tauri::command]
    fn save_app_preferences(
        app: tauri::AppHandle,
        save_mode: files::SaveMode,
        output_dir: PathBuf,
        language: preferences::Language,
        theme: preferences::Theme,
    ) -> Result<Preferences, super::messages::AppError> {
        update_app_preferences(&app, |preferences| {
            preferences.language = language;
            preferences.theme = theme;
            preferences.save_mode = save_mode;
            preferences.output_dir = output_dir;
        })
    }

    #[tauri::command]
    fn remove_asset_folder(
        app: tauri::AppHandle,
        path: String,
    ) -> Result<Preferences, super::messages::AppError> {
        update_app_preferences(&app, |preferences| {
            preferences.folders.retain(|folder| folder != &path);
        })
    }

    #[tauri::command]
    async fn read_preview(
        path: String,
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<tauri::ipc::Response, super::messages::AppError> {
        if !state
            .snapshots
            .lock()
            .map_err(|e| e.to_string())?
            .contains_key(Path::new(&path))
            && !state
                .output_previews
                .lock()
                .map_err(|error| error.to_string())?
                .contains(Path::new(&path))
        {
            return Err("Impor file terlebih dahulu.".into());
        }
        let queue = state.previews.clone();
        let cache_dir = preview_cache_dir(&app)?;
        tauri::async_runtime::spawn_blocking(move || {
            let _guard = queue.lock().map_err(|e| e.to_string())?;
            preview::read_cached(Path::new(&path), &cache_dir).map(tauri::ipc::Response::new)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(Into::into)
    }

    #[tauri::command]
    async fn get_preview_cache_size(
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<u64, super::messages::AppError> {
        let queue = state.previews.clone();
        let cache_dir = preview_cache_dir(&app)?;
        tauri::async_runtime::spawn_blocking(move || {
            let _guard = queue.lock().map_err(|e| e.to_string())?;
            preview::cache_size(&cache_dir)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(Into::into)
    }

    #[tauri::command]
    async fn clear_preview_cache(
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<u64, super::messages::AppError> {
        let queue = state.previews.clone();
        let cache_dir = preview_cache_dir(&app)?;
        tauri::async_runtime::spawn_blocking(move || {
            let _guard = queue.lock().map_err(|e| e.to_string())?;
            preview::clear_cache(&cache_dir)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(Into::into)
    }

    #[tauri::command]
    async fn save_metadata(
        request: SaveRequest,
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<SaveResponse, super::messages::AppError> {
        let _worker = lock_worker(&state)?;
        let snapshots = state.snapshots.clone();
        let preferences = load_app_preferences(app.clone())?;
        tauri::async_runtime::spawn_blocking(move || {
            if let Some(category) = &request.shutterstock_category {
                super::shutterstock::validate_save(category, &request.items)?;
            }
            let total = request.items.len();
            let mut results = Vec::new();
            for item in request.items {
                let expected = snapshots
                    .lock()
                    .map_err(|e| e.to_string())?
                    .get(Path::new(&item.path))
                    .cloned();
                let result = expected
                    .ok_or_else(|| "Impor ulang file sebelum menyimpan.".to_owned())
                    .and_then(|expected| {
                        let directory = if matches!(preferences.save_mode, files::SaveMode::Copy) {
                            Some(files::copy_directory(
                                &preferences.output_dir,
                                Path::new(&item.path),
                            )?)
                        } else {
                            None
                        };
                        files::save_one(
                            &item,
                            &preferences.save_mode,
                            directory.as_deref(),
                            &expected,
                        )
                    });
                let outcome = match result {
                    Ok(value) => {
                        if let Some(output) = &value.output_path {
                            if let Ok(stamp) = files::stamp(Path::new(output)) {
                                snapshots
                                    .lock()
                                    .map_err(|e| e.to_string())?
                                    .insert(PathBuf::from(output), stamp);
                            }
                        }
                        value
                    }
                    Err(error) => SaveResult {
                        path: item.path,
                        output_path: None,
                        backup_path: None,
                        metadata: None,
                        error: Some(error),
                    },
                };
                results.push(outcome);
                let _ = app.emit(
                    "save_progress",
                    serde_json::json!({"completed": results.len(), "total": total}),
                );
            }
            let csv_results = request
                .shutterstock_category
                .as_deref()
                .map(|category| super::shutterstock::export_saved(category, &results))
                .unwrap_or_default();
            Ok::<_, String>(SaveResponse {
                results,
                csv_results,
            })
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(Into::into)
    }

    #[tauri::command]
    fn load_ai_settings(
        app: tauri::AppHandle,
    ) -> Result<ai::AiSettingsView, super::messages::AppError> {
        ai::load_settings(&app).map_err(Into::into)
    }

    #[tauri::command]
    fn save_ai_settings(
        app: tauri::AppHandle,
        provider_settings: Vec<ai::ProviderSetting>,
        selected_provider: String,
        api_key: Option<String>,
        clear_api_key: bool,
        description_length: String,
        keyword_count: u8,
    ) -> Result<ai::AiSettingsView, super::messages::AppError> {
        ai::save_settings(
            &app,
            AiSettingsInput {
                provider_settings,
                selected_provider,
                api_key,
                clear_api_key,
                description_length,
                keyword_count,
            },
        )
        .map_err(Into::into)
    }

    #[tauri::command]
    fn save_generator_preferences(
        app: tauri::AppHandle,
        description_length: String,
        keyword_count: u8,
    ) -> Result<ai::AiSettingsView, super::messages::AppError> {
        ai::save_generator_preferences(&app, &description_length, keyword_count).map_err(Into::into)
    }

    #[tauri::command]
    async fn list_ai_models(
        draft: ProviderDraft,
    ) -> Result<Vec<ai::AiModelOption>, super::messages::AppError> {
        ai::list_models(draft).await.map_err(Into::into)
    }

    #[tauri::command]
    async fn test_ai_connection(draft: ProviderDraft) -> Result<(), super::messages::AppError> {
        ai::test_connection(draft).await.map_err(Into::into)
    }

    #[tauri::command]
    async fn generate_title_description(
        app: tauri::AppHandle,
        request: AiGenerationRequest,
        state: State<'_, AppState>,
    ) -> Result<ai::GeneratedTitleDescription, super::messages::AppError> {
        let preview = ai_preview(&request.path, &app, &state).await?;
        ai::generate_title_description(&app, request, preview)
            .await
            .map_err(Into::into)
    }

    #[tauri::command]
    async fn generate_keywords(
        app: tauri::AppHandle,
        request: AiGenerationRequest,
        state: State<'_, AppState>,
    ) -> Result<ai::GeneratedKeywords, super::messages::AppError> {
        let preview = ai_preview(&request.path, &app, &state).await?;
        ai::generate_keywords(&app, request, preview)
            .await
            .map_err(Into::into)
    }

    async fn ai_preview(
        path: &str,
        app: &tauri::AppHandle,
        state: &AppState,
    ) -> Result<Option<Vec<u8>>, String> {
        if !state
            .snapshots
            .lock()
            .map_err(|e| e.to_string())?
            .contains_key(Path::new(path))
        {
            return Err("Impor aset terlebih dahulu.".into());
        }
        let path = PathBuf::from(path);
        let queue = state.previews.clone();
        let cache_dir = preview_cache_dir(app)?;
        tauri::async_runtime::spawn_blocking(move || {
            let _guard = queue.lock().map_err(|e| e.to_string())?;
            Ok(preview::read_cached(&path, &cache_dir).ok())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    pub fn run() {
        tauri::Builder::default()
            .manage(AppState::default())
            .plugin(tauri_plugin_dialog::init())
            .invoke_handler(tauri::generate_handler![
                inspect_assets,
                inspect_output_assets,
                load_app_preferences,
                open_output_directory,
                save_app_preferences,
                remove_asset_folder,
                read_preview,
                get_preview_cache_size,
                clear_preview_cache,
                save_metadata,
                load_ai_settings,
                save_ai_settings,
                save_generator_preferences,
                list_ai_models,
                test_ai_connection,
                generate_title_description,
                generate_keywords
            ])
            .setup(|app| {
                // xmp_toolkit initializes its SDK lazily. Surface initialization errors at startup.
                xmp_toolkit::XmpMeta::new()?;
                let _ = app.get_webview_window("main");
                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("Gagal menjalankan Stock Asset Metadata");
    }
}

#[cfg(feature = "desktop")]
pub use desktop::run;
