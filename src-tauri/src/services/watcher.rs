use crate::adapters::ConfigAdapterRegistry;
use crate::db::Repository;
use crate::error::AppResult;
use crate::models::ReconciliationConflict;
use crate::services::ReconciliationService;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct FileWatcherService {
    _watcher: RecommendedWatcher,
}

impl FileWatcherService {
    pub fn start(app: AppHandle, db: Arc<Mutex<Connection>>) -> AppResult<Self> {
        let app_handle = app.clone();
        let db_arc = db.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if crate::services::EnableDisableService::is_writing() {
                    return;
                }
                if let Ok(event) = res {
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                    ) {
                        std::thread::sleep(Duration::from_millis(500));
                        if let Ok(conn) = db_arc.lock() {
                            if let Ok(conflicts) = ReconciliationService::detect_conflicts(&conn) {
                                if !conflicts.is_empty() {
                                    let _ = app_handle.emit("reconciliation-conflicts", &conflicts);
                                }
                            }
                        }
                    }
                }
            },
            Config::default(),
        )?;

        Self::watch_all_paths(&mut watcher, &db)?;

        Ok(Self { _watcher: watcher })
    }

    pub fn refresh_watches(_watcher: &mut RecommendedWatcher, _db: &Connection) -> AppResult<()> {
        // Watcher paths are registered at startup; restart app after adding projects.
        Ok(())
    }

    fn watch_all_paths(watcher: &mut RecommendedWatcher, db: &Arc<Mutex<Connection>>) -> AppResult<()> {
        let watched: Vec<PathBuf> = if let Ok(conn) = db.lock() {
            Repository::list_watched_projects(&conn)?
                .into_iter()
                .map(|p| PathBuf::from(p.path))
                .collect()
        } else {
            vec![]
        };

        for adapter in ConfigAdapterRegistry::all_adapters_including_projects(&watched) {
            for path in adapter.file_paths_watched() {
                if path.exists() {
                    if let Some(parent) = path.parent() {
                        let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
                    }
                    let _ = watcher.watch(&path, RecursiveMode::NonRecursive);
                } else if let Some(parent) = path.parent() {
                    let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
                }
            }
        }
        Ok(())
    }
}

pub fn detect_and_emit(app: &AppHandle, conn: &Connection) -> AppResult<Vec<ReconciliationConflict>> {
    let conflicts = ReconciliationService::detect_conflicts(conn)?;
    if !conflicts.is_empty() {
        let _ = app.emit("reconciliation-conflicts", &conflicts);
    }
    Ok(conflicts)
}
