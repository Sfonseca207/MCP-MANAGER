use super::{
    extract_servers_from_value, read_json_file, set_servers_in_value, write_json_file, ConfigAdapter,
    ConfigTarget,
};
use crate::error::AppResult;
use crate::models::{McpDefinition, ScopeType};
use crate::utils::home_dir;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct ClaudeDesktopAdapter;

impl ClaudeDesktopAdapter {
    fn config_path() -> AppResult<PathBuf> {
        Ok(home_dir()?
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json"))
    }
}

impl ConfigAdapter for ClaudeDesktopAdapter {
    fn target(&self) -> ConfigTarget {
        ConfigTarget {
            client: crate::models::ClientType::ClaudeDesktop,
            scope_type: ScopeType::Global,
            scope_path: None,
            file_path: Self::config_path().unwrap_or_default(),
            servers_key: "mcpServers".into(),
            use_windsurf_format: false,
        }
    }

    fn read_mcp_servers(&self) -> AppResult<BTreeMap<String, McpDefinition>> {
        let path = Self::config_path()?;
        let root = read_json_file(&path)?;
        extract_servers_from_value(&root, "mcpServers")
    }

    fn write_mcp_servers(&self, servers: &BTreeMap<String, McpDefinition>) -> AppResult<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let root = read_json_file(&path)?;
        let updated = set_servers_in_value(root, "mcpServers", servers, false);
        write_json_file(&path, &updated)
    }
}
