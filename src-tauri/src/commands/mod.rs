use crate::db::Repository;
use crate::error::AppResult;
use crate::models::{
    BackupRecord, BootstrapSummary, CatalogEntry, ClientType, CreateMcpInput,
    ReconciliationConflict, StatusSummary, UpdateMcpInput, WatchedProject,
};
use crate::services::{
    BackupService, BootstrapService, CatalogService, EnableDisableService, ReconciliationService,
};
use crate::services::watcher::detect_and_emit;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_app_info() -> AppResult<serde_json::Value> {
    Ok(serde_json::json!({
        "name": "MCP Manager",
        "version": env!("CARGO_PKG_VERSION"),
        "identifier": "com.samuelfonseca.mcpmanager"
    }))
}

#[tauri::command]
pub fn bootstrap_if_needed(state: State<'_, AppState>) -> AppResult<BootstrapSummary> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    BootstrapService::bootstrap_if_needed(&conn)
}

#[tauri::command]
pub fn list_catalog(state: State<'_, AppState>) -> AppResult<Vec<CatalogEntry>> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    CatalogService::list(&conn)
}

#[tauri::command]
pub fn get_status_summary(state: State<'_, AppState>) -> AppResult<StatusSummary> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    CatalogService::status_summary(&conn)
}

#[tauri::command]
pub fn enable_mcp(
    state: State<'_, AppState>,
    entry_id: String,
    clients: Vec<String>,
) -> AppResult<Vec<String>> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    let client_types: Vec<ClientType> = clients
        .iter()
        .filter_map(|c| ClientType::from_str(c))
        .collect();
    EnableDisableService::enable(&conn, &entry_id, &client_types)
}

#[tauri::command]
pub fn disable_mcp(
    state: State<'_, AppState>,
    entry_id: String,
    clients: Vec<String>,
) -> AppResult<Vec<String>> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    let client_types: Vec<ClientType> = clients
        .iter()
        .filter_map(|c| ClientType::from_str(c))
        .collect();
    EnableDisableService::disable(&conn, &entry_id, &client_types)
}

#[tauri::command]
pub fn create_mcp(state: State<'_, AppState>, input: CreateMcpInput) -> AppResult<String> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    Repository::create_manual_entry(&conn, &input)
}

#[tauri::command]
pub fn update_mcp(state: State<'_, AppState>, input: UpdateMcpInput) -> AppResult<Vec<String>> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    CatalogService::update(&conn, &input)
}

#[tauri::command]
pub fn hard_delete_mcp(state: State<'_, AppState>, entry_id: String) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    Repository::hard_delete(&conn, &entry_id)
}

#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> AppResult<Vec<BackupRecord>> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    Repository::list_backups(&conn)
}

#[tauri::command]
pub fn restore_backup(state: State<'_, AppState>, backup_id: String) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    BackupService::restore_backup(&conn, &backup_id)
}

#[tauri::command]
pub fn detect_conflicts(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Vec<ReconciliationConflict>> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    detect_and_emit(&app, &conn)
}

#[tauri::command]
pub fn resolve_reconciliation(
    state: State<'_, AppState>,
    conflict: ReconciliationConflict,
    action: String,
) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    match action.as_str() {
        "import" => ReconciliationService::import_external(&conn, &conflict),
        "mark_disabled" => ReconciliationService::mark_external_removed(&conn, &conflict),
        "ignore" => Ok(()),
        _ => Err("Unknown action".into()),
    }
}

#[tauri::command]
pub fn rescan_configs(state: State<'_, AppState>) -> AppResult<BootstrapSummary> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    BootstrapService::rescan_all(&conn)
}

#[tauri::command]
pub fn add_watched_project(
    state: State<'_, AppState>,
    path: String,
    label: Option<String>,
) -> AppResult<WatchedProject> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    Repository::add_watched_project(&conn, &path, label.as_deref())
}

#[tauri::command]
pub fn list_watched_projects(state: State<'_, AppState>) -> AppResult<Vec<WatchedProject>> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    Repository::list_watched_projects(&conn)
}

#[tauri::command]
pub fn remove_watched_project(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| "Database lock error")?;
    Repository::remove_watched_project(&conn, &id)
}

#[tauri::command]
pub fn mask_secret_value(value: String) -> String {
    crate::utils::mask_secret(&value)
}
