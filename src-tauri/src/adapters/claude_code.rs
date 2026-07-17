use super::{
    extract_servers_from_value, read_json_file, set_servers_in_value, write_json_file, ConfigAdapter,
    ConfigTarget,
};
use crate::error::AppResult;
use crate::models::{McpDefinition, ScopeType};
use crate::utils::home_dir;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct ClaudeCodeGlobalAdapter;

impl ClaudeCodeGlobalAdapter {
    fn config_path() -> AppResult<PathBuf> {
        Ok(home_dir()?.join(".claude.json"))
    }
}

impl ConfigAdapter for ClaudeCodeGlobalAdapter {
    fn target(&self) -> ConfigTarget {
        ConfigTarget {
            client: crate::models::ClientType::ClaudeCode,
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
        let root = read_json_file(&path)?;
        let updated = set_servers_in_value(root, "mcpServers", servers, false);
        write_json_file(&path, &updated)
    }
}

pub struct ClaudeCodeProjectAdapter {
    project_path: PathBuf,
}

impl ClaudeCodeProjectAdapter {
    pub fn new(project_path: &std::path::Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
        }
    }

    fn claude_json_path() -> AppResult<PathBuf> {
        Ok(home_dir()?.join(".claude.json"))
    }

    fn mcp_json_path(&self) -> PathBuf {
        self.project_path.join(".mcp.json")
    }
}

impl ConfigAdapter for ClaudeCodeProjectAdapter {
    fn target(&self) -> ConfigTarget {
        ConfigTarget {
            client: crate::models::ClientType::ClaudeCode,
            scope_type: ScopeType::Project,
            scope_path: Some(self.project_path.clone()),
            file_path: Self::claude_json_path().unwrap_or_default(),
            servers_key: "mcpServers".into(),
            use_windsurf_format: false,
        }
    }

    fn read_mcp_servers(&self) -> AppResult<BTreeMap<String, McpDefinition>> {
        let mut servers = BTreeMap::new();

        // Read from .mcp.json if exists
        let mcp_path = self.mcp_json_path();
        if mcp_path.exists() {
            let root = read_json_file(&mcp_path)?;
            servers.extend(extract_servers_from_value(&root, "mcpServers")?);
        }

        // Read from projects block in ~/.claude.json
        let claude_path = Self::claude_json_path()?;
        let root = read_json_file(&claude_path)?;
        let project_key = self.project_path.to_string_lossy().to_string();
        if let Some(projects) = root.get("projects").and_then(|p| p.as_object()) {
            if let Some(project) = projects.get(&project_key) {
                if let Ok(project_servers) =
                    extract_servers_from_value(project, "mcpServers")
                {
                    servers.extend(project_servers);
                }
            }
        }

        Ok(servers)
    }

    fn write_mcp_servers(&self, servers: &BTreeMap<String, McpDefinition>) -> AppResult<()> {
        // Write to .mcp.json for project-shared config
        let mcp_path = self.mcp_json_path();
        if let Some(parent) = mcp_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let root = read_json_file(&mcp_path)?;
        let updated = set_servers_in_value(root, "mcpServers", servers, false);
        write_json_file(&mcp_path, &updated)
    }

    fn file_paths_watched(&self) -> Vec<PathBuf> {
        vec![Self::claude_json_path().unwrap_or_default(), self.mcp_json_path()]
    }
}
