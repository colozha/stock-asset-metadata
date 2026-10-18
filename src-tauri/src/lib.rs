pub mod files;
pub mod messages;
pub mod metadata;
pub mod preferences;
pub mod preview;
pub mod svg;

#[cfg(feature = "desktop")]
mod ai;

#[cfg(feature = "desktop")]
mod desktop {
    use super::{
        ai::{self, AiGenerationRequest, AiSettingsInput, ProviderDraft},
        files::{self, FileStamp, SaveRequest, SaveResult},
        preferences::{self, Preferences},
        preview,
    };
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };
    use tauri::{Emitter, Manager, State};

    #[derive(Default)]
    struct AppState {
        snapshots: Arc<Mutex<HashMap<PathBuf, FileStamp>>>,
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
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<files::Inspection, super::messages::AppError> {
        let _worker = lock_worker(&state)?;
        let mut preferences = load_app_preferences(app.clone())?;
        let known = preferences.folders.clone();
        let (report, stamps) = tauri::async_runtime::spawn_blocking(move || {
            files::inspect_with_folders(paths, &known)
        })
        .await
        .map_err(|e| e.to_string())?;
        let mut snapshots = state.snapshots.lock().map_err(|e| e.to_string())?;
        for (path, stamp) in stamps {
            if refresh.unwrap_or(false) {
                snapshots.insert(path, stamp);
            } else {
                snapshots.entry(path).or_insert(stamp);
            }
        }
        drop(snapshots);
        if !refresh.unwrap_or(false) {
            for folder in &report.folders {
                if !preferences.folders.contains(&folder.path) {
                    preferences.folders.push(folder.path.clone());
                }
            }
            preferences::save(&preferences_path(&app)?, &preferences)?;
        }
        Ok(report)
    }

    fn preferences_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        Ok(app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join("app-preferences.json"))
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
        let documents = app
            .path()
            .document_dir()
            .or_else(|_| app.path().home_dir().map(|home| home.join("Documents")))
            .map_err(|e| format!("Direktori Documents tidak tersedia: {e}"))?;
        preferences::load(&preferences_path(&app)?, &documents).map_err(Into::into)
    }

    #[tauri::command]
    fn save_app_preferences(
        app: tauri::AppHandle,
        save_mode: files::SaveMode,
        output_dir: PathBuf,
        language: preferences::Language,
        theme: preferences::Theme,
    ) -> Result<Preferences, super::messages::AppError> {
        let mut preferences = load_app_preferences(app.clone())?;
        preferences.language = language;
        preferences.theme = theme;
        preferences.save_mode = save_mode;
        preferences.output_dir = output_dir;
        preferences::save(&preferences_path(&app)?, &preferences)?;
        Ok(preferences)
    }

    #[tauri::command]
    fn remove_asset_folder(
        app: tauri::AppHandle,
        path: String,
    ) -> Result<Preferences, super::messages::AppError> {
        let mut preferences = load_app_preferences(app.clone())?;
        preferences.folders.retain(|folder| folder != &path);
        preferences::save(&preferences_path(&app)?, &preferences)?;
        Ok(preferences)
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
    ) -> Result<Vec<SaveResult>, super::messages::AppError> {
        let _worker = lock_worker(&state)?;
        let snapshots = state.snapshots.clone();
        let preferences = load_app_preferences(app.clone())?;
        tauri::async_runtime::spawn_blocking(move || {
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
            Ok::<_, String>(results)
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
                load_app_preferences,
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
