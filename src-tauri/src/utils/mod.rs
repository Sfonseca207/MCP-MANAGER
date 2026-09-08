use crate::error::{AppError, AppResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn atomic_write_json(path: &Path, content: &str) -> AppResult<()> {
    // Validate before touching the filesystem so an invalid payload never
    // leaves a stray temp file next to the client's config.
    serde_json::from_str::<serde_json::Value>(content)
        .map_err(|e| AppError::Message(format!("Invalid JSON: {e}")))?;

    let parent = path
        .parent()
        .ok_or_else(|| "Invalid file path".to_string())?;
    fs::create_dir_all(parent)?;

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_path = parent.join(format!(
        ".{}.{}.{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file"),
        std::process::id(),
        unique
    ));

    let result = (|| -> AppResult<()> {
        {
            use std::io::Write;
            let mut file = fs::File::create(&temp_path)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
        }
        fs::rename(&temp_path, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

pub fn definition_hash(definition: &crate::models::McpDefinition) -> String {
    use sha2::{Digest, Sha256};
    let json = definition.to_json_value();
    let canonical = serde_json::to_string(&json).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn servers_hash(servers: &std::collections::BTreeMap<String, crate::models::McpDefinition>) -> String {
    use sha2::{Digest, Sha256};
    let mut entries: Vec<_> = servers.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let payload = serde_json::to_string(&entries).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn app_data_dir() -> AppResult<PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| "Could not resolve data directory".to_string())?
        .join("com.samuelfonseca.mcpmanager");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn backups_dir() -> AppResult<PathBuf> {
    let dir = app_data_dir()?.join("backups");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn db_path() -> AppResult<PathBuf> {
    Ok(app_data_dir()?.join("catalog.db"))
}

pub fn home_dir() -> AppResult<PathBuf> {
    dirs::home_dir().ok_or_else(|| AppError::Message("Could not resolve home directory".into()))
}

pub fn expand_tilde(path: &str) -> AppResult<PathBuf> {
    if path.starts_with("~/") {
        Ok(home_dir()?.join(&path[2..]))
    } else {
        Ok(PathBuf::from(path))
    }
}

pub fn mask_secret(value: &str) -> String {
    // Fixed-width output so the mask never reveals the secret's length,
    // char-based slicing so multibyte UTF-8 can't panic.
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 8 {
        return "********".into();
    }
    let head: String = chars[..2].iter().collect();
    let tail: String = chars[chars.len() - 2..].iter().collect();
    format!("{head}********{tail}")
}
