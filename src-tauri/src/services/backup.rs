use crate::adapters::ConfigAdapter;
use crate::db::Repository;
use crate::error::AppResult;
use crate::utils::backups_dir;
use chrono::Utc;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

/// Backups kept per target config file; older ones are pruned automatically.
const BACKUPS_KEEP_PER_FILE: usize = 20;

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
        let scope_path_str = target
            .scope_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        Self::write_backup(
            conn,
            target.client,
            target.scope_type,
            scope_path_str.as_deref(),
            &file_path.to_string_lossy(),
            &content,
            reason,
        )
    }

    fn write_backup(
        conn: &Connection,
        client: crate::models::ClientType,
        scope_type: crate::models::ScopeType,
        scope_path: Option<&str>,
        file_path: &str,
        content: &str,
        reason: &str,
    ) -> AppResult<String> {
        let backup_dir = backups_dir()?
            .join(client.as_str())
            .join(scope_type.as_str());
        fs::create_dir_all(&backup_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("config.json");
        let backup_path = backup_dir.join(format!("{timestamp}_{file_name}"));
        fs::write(&backup_path, content)?;

        let id = Repository::insert_backup(
            conn,
            client,
            scope_type,
            scope_path,
            file_path,
            &backup_path.to_string_lossy(),
            reason,
        )?;

        for stale in Repository::prune_backups(conn, file_path, BACKUPS_KEEP_PER_FILE)? {
            let _ = fs::remove_file(stale);
        }
        Ok(id)
    }

    pub fn restore_backup(conn: &Connection, backup_id: &str) -> AppResult<()> {
        let backups = Repository::list_backups(conn)?;
        let backup = backups
            .into_iter()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| "Backup not found".to_string())?;

        // Defense in depth: only restore from files inside our backups directory.
        let backups_root = backups_dir()?.canonicalize()?;
        let backup_file = Path::new(&backup.backup_path)
            .canonicalize()
            .map_err(|_| "Backup file not found".to_string())?;
        if !backup_file.starts_with(&backups_root) {
            return Err("Backup path is outside the backups directory".to_string().into());
        }

        let content = fs::read_to_string(&backup_file)?;
        let path = Path::new(&backup.file_path);

        // Safety net: snapshot the current file so a restore is itself reversible.
        if path.exists() {
            let current = fs::read_to_string(path)?;
            Self::write_backup(
                conn,
                backup.client,
                backup.scope_type,
                backup.scope_path.as_deref(),
                &backup.file_path,
                &current,
                "pre_restore",
            )?;
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        crate::utils::atomic_write_json(path, &content)?;
        Ok(())
    }
}
