use rusqlite::Connection;
use std::sync::Mutex;

use crate::services::FileWatcherService;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub _watcher: Mutex<Option<FileWatcherService>>,
}
