use super::{ClientType, McpDefinition, ScopeType, SourceType, TransportType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub display_name: String,
    pub transport_type: TransportType,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
    pub client_origin: ClientType,
    pub scope_type: ScopeType,
    pub scope_path: Option<String>,
    pub source: SourceType,
    pub definition_hash: String,
    pub created_at: String,
    pub updated_at: String,
    pub client_states: Vec<ClientState>,
}

impl CatalogEntry {
    pub fn to_definition(&self) -> McpDefinition {
        McpDefinition {
            name: self.display_name.clone(),
            transport_type: self.transport_type,
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientState {
    pub client: ClientType,
    pub enabled: bool,
    pub last_enabled_at: Option<String>,
    pub last_disabled_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: String,
    pub client: ClientType,
    pub scope_type: ScopeType,
    pub scope_path: Option<String>,
    pub file_path: String,
    pub backup_path: String,
    pub created_at: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapSummary {
    pub claude_code_count: u32,
    pub cursor_count: u32,
    pub claude_desktop_count: u32,
    pub vscode_count: u32,
    pub windsurf_count: u32,
    pub total: u32,
    pub first_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSummary {
    pub total_catalog: u32,
    pub active_by_client: BTreeMap<String, u32>,
    pub estimated_tools_by_client: BTreeMap<String, u32>,
    pub cursor_tool_limit: u32,
    pub cursor_over_limit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationKind {
    NewExternal,
    RemovedExternal,
    DefinitionChanged,
    NameCollision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationConflict {
    pub id: String,
    pub kind: ReconciliationKind,
    pub client: ClientType,
    pub scope_type: ScopeType,
    pub scope_path: Option<String>,
    pub server_name: String,
    pub catalog_entry_id: Option<String>,
    pub external_definition: Option<McpDefinition>,
    pub detected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMcpInput {
    pub display_name: String,
    pub transport_type: TransportType,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
    pub client_origin: ClientType,
    pub scope_type: ScopeType,
    pub scope_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMcpInput {
    pub id: String,
    pub display_name: String,
    pub transport_type: TransportType,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedProject {
    pub id: String,
    pub path: String,
    pub label: Option<String>,
    pub created_at: String,
}
