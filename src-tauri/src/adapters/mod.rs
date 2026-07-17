use crate::error::{AppError, AppResult};
use crate::models::{ClientType, McpDefinition, ScopeType};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub mod claude_code;
pub mod claude_desktop;
pub mod cursor;
pub mod vscode;
pub mod windsurf;

pub use claude_code::*;
pub use claude_desktop::*;
pub use cursor::*;
pub use vscode::*;
pub use windsurf::*;

#[derive(Debug, Clone)]
pub struct ConfigTarget {
    pub client: ClientType,
    pub scope_type: ScopeType,
    pub scope_path: Option<PathBuf>,
    pub file_path: PathBuf,
    pub servers_key: String,
    pub use_windsurf_format: bool,
}

impl ConfigTarget {
    pub fn label(&self) -> String {
        match (&self.scope_type, &self.scope_path) {
            (ScopeType::Global, _) => format!("{} (global)", self.client.label()),
            (ScopeType::Project, Some(path)) => {
                format!("{} ({})", self.client.label(), path.display())
            }
            (ScopeType::Project, None) => format!("{} (project)", self.client.label()),
        }
    }
}

pub trait ConfigAdapter: Send + Sync {
    fn target(&self) -> ConfigTarget;
    fn read_mcp_servers(&self) -> AppResult<BTreeMap<String, McpDefinition>>;
    fn write_mcp_servers(&self, servers: &BTreeMap<String, McpDefinition>) -> AppResult<()>;
    fn file_paths_watched(&self) -> Vec<PathBuf> {
        vec![self.target().file_path]
    }
}

pub struct ConfigAdapterRegistry;

impl ConfigAdapterRegistry {
    pub fn all_global_adapters() -> Vec<Box<dyn ConfigAdapter>> {
        vec![
            Box::new(ClaudeCodeGlobalAdapter),
            Box::new(CursorGlobalAdapter),
            Box::new(ClaudeDesktopAdapter),
            Box::new(VsCodeUserAdapter::new()),
            Box::new(WindsurfAdapter),
        ]
    }

    pub fn adapter_for(client: ClientType, scope_type: ScopeType, scope_path: Option<&Path>) -> AppResult<Box<dyn ConfigAdapter>> {
        match (client, scope_type) {
            (ClientType::ClaudeCode, ScopeType::Global) => {
                Ok(Box::new(ClaudeCodeGlobalAdapter))
            }
            (ClientType::ClaudeCode, ScopeType::Project) => {
                let path = scope_path.ok_or_else(|| "Project path required".to_string())?;
                Ok(Box::new(ClaudeCodeProjectAdapter::new(path)))
            }
            (ClientType::Cursor, ScopeType::Global) => Ok(Box::new(CursorGlobalAdapter)),
            (ClientType::Cursor, ScopeType::Project) => {
                let path = scope_path.ok_or_else(|| "Project path required".to_string())?;
                Ok(Box::new(CursorProjectAdapter::new(path)))
            }
            (ClientType::ClaudeDesktop, ScopeType::Global) => Ok(Box::new(ClaudeDesktopAdapter)),
            (ClientType::VsCode, ScopeType::Global) => Ok(Box::new(VsCodeUserAdapter::new())),
            (ClientType::VsCode, ScopeType::Project) => {
                let path = scope_path.ok_or_else(|| "Project path required".to_string())?;
                Ok(Box::new(VsCodeWorkspaceAdapter::new(path)))
            }
            (ClientType::Windsurf, ScopeType::Global) => Ok(Box::new(WindsurfAdapter)),
            (ClientType::Windsurf, ScopeType::Project) => {
                Err(AppError::Message("Windsurf only supports global scope".into()))
            }
            (ClientType::ClaudeDesktop, ScopeType::Project) => {
                Err(AppError::Message("Claude Desktop only supports global scope".into()))
            }
        }
    }

    pub fn all_adapters_including_projects(watched_projects: &[PathBuf]) -> Vec<Box<dyn ConfigAdapter>> {
        let mut adapters = Self::all_global_adapters();
        for project in watched_projects {
            adapters.push(Box::new(ClaudeCodeProjectAdapter::new(project)));
            adapters.push(Box::new(CursorProjectAdapter::new(project)));
            adapters.push(Box::new(VsCodeWorkspaceAdapter::new(project)));
        }
        adapters
    }
}

pub fn read_json_file(path: &Path) -> AppResult<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = std::fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    Ok(serde_json::from_str(&content)?)
}

pub fn write_json_file(path: &Path, value: &serde_json::Value) -> AppResult<()> {
    let content = serde_json::to_string_pretty(value)?;
    crate::utils::atomic_write_json(path, &content)
}

pub fn extract_servers_from_value(
    value: &serde_json::Value,
    key: &str,
) -> AppResult<BTreeMap<String, McpDefinition>> {
    let servers_value = value.get(key).cloned().unwrap_or(serde_json::json!({}));
    crate::models::parse_mcp_servers_map(&servers_value)
}

pub fn set_servers_in_value(
    mut root: serde_json::Value,
    key: &str,
    servers: &BTreeMap<String, McpDefinition>,
    use_windsurf: bool,
) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    for (name, def) in servers {
        let json = if use_windsurf {
            def.to_windsurf_json_value()
        } else {
            def.to_json_value()
        };
        obj.insert(name.clone(), json);
    }
    if let Some(root_obj) = root.as_object_mut() {
        root_obj.insert(key.to_string(), serde_json::Value::Object(obj));
    } else {
        root = serde_json::json!({ key: obj });
    }
    root
}
