mod adapters;
mod commands;
mod db;
mod error;
mod models;
mod services;
mod state;
mod utils;

use db::run_migrations;
use error::AppResult;
use rusqlite::Connection;
use services::{BootstrapService, FileWatcherService};
use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use utils::db_path;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let conn = init_database().map_err(|e| e.to_string())?;
            let summary = BootstrapService::bootstrap_if_needed(&conn).map_err(|e| e.to_string())?;
            let _ = summary;

            let app_state = AppState {
                db: Mutex::new(conn),
                _watcher: Mutex::new(None),
            };
            app.manage(app_state);

            let handle = app.handle().clone();
            let db_path = db_path().map_err(|e| e.to_string())?;
            let conn_for_watcher =
                Connection::open(&db_path).map_err(|e| e.to_string())?;
            let db_arc = Arc::new(Mutex::new(conn_for_watcher));

            if let Some(state) = app.try_state::<AppState>() {
                let watcher = FileWatcherService::start(handle, db_arc).map_err(|e| e.to_string())?;
                *state
                    ._watcher
                    .lock()
                    .map_err(|_| "Watcher lock error".to_string())? = Some(watcher);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::bootstrap_if_needed,
            commands::list_catalog,
            commands::get_status_summary,
            commands::enable_mcp,
            commands::disable_mcp,
            commands::create_mcp,
            commands::update_mcp,
            commands::hard_delete_mcp,
            commands::list_backups,
            commands::restore_backup,
            commands::detect_conflicts,
            commands::resolve_reconciliation,
            commands::rescan_configs,
            commands::add_watched_project,
            commands::list_watched_projects,
            commands::remove_watched_project,
            commands::mask_secret_value,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_database() -> AppResult<Connection> {
    let path = db_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use crate::models::{McpDefinition, TransportType, parse_mcp_servers_map};
    use crate::utils::{atomic_write_json, definition_hash};
    use std::collections::BTreeMap;
    use std::fs;

    #[test]
    fn parse_mcp_servers_from_cursor_format() {
        let json: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/cursor-mcp.json")).unwrap();
        let servers = parse_mcp_servers_map(&json["mcpServers"]).unwrap();
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("cursor-server"));
    }

    #[test]
    fn claude_json_patch_preserves_other_keys() {
        let fixture = include_str!("../../tests/fixtures/claude.json");
        let root: serde_json::Value = serde_json::from_str(fixture).unwrap();

        let mut servers = BTreeMap::new();
        servers.insert(
            "new-server".to_string(),
            McpDefinition {
                name: "new-server".to_string(),
                transport_type: TransportType::Stdio,
                command: Some("echo".into()),
                args: Some(vec!["hello".into()]),
                env: None,
                url: None,
                headers: None,
            },
        );

        let updated = crate::adapters::set_servers_in_value(
            root,
            "mcpServers",
            &servers,
            false,
        );

        assert_eq!(updated["preserved_root_key"], "should remain");
        assert_eq!(updated["onboarding"]["completed"], true);
        assert!(updated["mcpServers"]["new-server"].is_object());
        assert!(updated["projects"]["/Users/test/project"]["mcpServers"]["project-server"].is_object());
    }

    #[test]
    fn atomic_write_validates_json() {
        let dir = std::env::temp_dir().join("mcp-manager-test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.json");

        let result = atomic_write_json(&path, "{ invalid json }");
        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn definition_hash_is_stable() {
        let def = McpDefinition {
            name: "test".into(),
            transport_type: TransportType::Stdio,
            command: Some("npx".into()),
            args: Some(vec!["-y".into(), "pkg".into()]),
            env: None,
            url: None,
            headers: None,
        };
        let h1 = definition_hash(&def);
        let h2 = definition_hash(&def);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }
}
