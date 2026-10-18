pub mod files;
pub mod metadata;
pub mod preview;
pub mod svg;

#[cfg(feature = "desktop")]
mod desktop {
    use super::{
        files::{self, Asset, FileStamp, SaveRequest, SaveResult},
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
        state: State<'_, AppState>,
    ) -> Result<Vec<Asset>, String> {
        let _worker = lock_worker(&state)?;
        let (assets, stamps) = tauri::async_runtime::spawn_blocking(move || files::inspect(paths))
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
        Ok(assets)
    }

    #[tauri::command]
    async fn read_preview(
        path: String,
        state: State<'_, AppState>,
    ) -> Result<tauri::ipc::Response, String> {
        if !state
            .snapshots
            .lock()
            .map_err(|e| e.to_string())?
            .contains_key(Path::new(&path))
        {
            return Err("Impor file terlebih dahulu.".into());
        }
        let queue = state.previews.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let _guard = queue.lock().map_err(|e| e.to_string())?;
            preview::read(Path::new(&path)).map(tauri::ipc::Response::new)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tauri::command]
    async fn save_metadata(
        request: SaveRequest,
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<Vec<SaveResult>, String> {
        let _worker = lock_worker(&state)?;
        let snapshots = state.snapshots.clone();
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
                        files::save_one(
                            &item,
                            &request.mode,
                            request.output_dir.as_deref().map(Path::new),
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
            Ok(results)
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
                read_preview,
                save_metadata
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
