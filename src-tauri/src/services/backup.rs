use crate::adapters::ConfigAdapter;
use crate::db::Repository;
use crate::error::AppResult;
use crate::utils::backups_dir;
use chrono::Utc;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

pub struct BackupService;

impl BackupService {
    pub fn create_backup(
        conn: &Connection,
        adapter: &dyn ConfigAdapter,
        reason: &str,
    ) -> AppResult<String> {
        let target = adapter.target();
        let file_path = &target.file_path;

        if !file_path.exists() {
            return Ok(String::new());
        }

        let content = fs::read_to_string(file_path)?;
        let backup_dir = backups_dir()?
            .join(target.client.as_str())
            .join(target.scope_type.as_str());
        fs::create_dir_all(&backup_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("config.json");
        let backup_path = backup_dir.join(format!("{timestamp}_{file_name}"));
        fs::write(&backup_path, content)?;

        let scope_path_str = target
            .scope_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        Repository::insert_backup(
            conn,
            target.client,
            target.scope_type,
            scope_path_str.as_deref(),
            &file_path.to_string_lossy(),
            &backup_path.to_string_lossy(),
            reason,
        )
    }

    pub fn restore_backup(conn: &Connection, backup_id: &str) -> AppResult<()> {
        let backups = Repository::list_backups(conn)?;
        let backup = backups
            .into_iter()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| "Backup not found".to_string())?;

        let content = fs::read_to_string(&backup.backup_path)?;
        let path = Path::new(&backup.file_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        crate::utils::atomic_write_json(path, &content)?;
        Ok(())
    }
}
