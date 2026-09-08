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

/// Splits the merged project-scope server map by provenance: servers that were
/// read from the private `projects` block of ~/.claude.json (and are not also
/// in .mcp.json) go back to the private block; everything else — including new
/// servers — goes to the project-shared .mcp.json.
pub fn split_project_servers(
    servers: &BTreeMap<String, McpDefinition>,
    existing_shared: &BTreeMap<String, McpDefinition>,
    existing_private: &BTreeMap<String, McpDefinition>,
) -> (
    BTreeMap<String, McpDefinition>,
    BTreeMap<String, McpDefinition>,
) {
    let mut shared_out = BTreeMap::new();
    let mut private_out = BTreeMap::new();
    for (name, def) in servers {
        if existing_private.contains_key(name) && !existing_shared.contains_key(name) {
            private_out.insert(name.clone(), def.clone());
        } else {
            shared_out.insert(name.clone(), def.clone());
        }
    }
    (shared_out, private_out)
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
        // Servers read from the private `projects` block of ~/.claude.json must be
        // written back there — never copied into .mcp.json, which is typically
        // committed to the project repo and would leak env secrets.
        let mcp_path = self.mcp_json_path();
        let claude_path = Self::claude_json_path()?;
        let project_key = self.project_path.to_string_lossy().to_string();

        let mcp_root = read_json_file(&mcp_path)?;
        let existing_shared = extract_servers_from_value(&mcp_root, "mcpServers")?;

        let claude_root = read_json_file(&claude_path)?;
        let existing_private = claude_root
            .get("projects")
            .and_then(|p| p.as_object())
            .and_then(|p| p.get(&project_key))
            .map(|project| extract_servers_from_value(project, "mcpServers"))
            .transpose()?
            .unwrap_or_default();

        let (shared_out, private_out) =
            split_project_servers(servers, &existing_shared, &existing_private);

        if !existing_private.is_empty() || !private_out.is_empty() {
            let mut root = claude_root;
            let root_obj = root
                .as_object_mut()
                .ok_or_else(|| "~/.claude.json root is not an object".to_string())?;
            let projects_obj = root_obj
                .entry("projects")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or_else(|| "~/.claude.json projects is not an object".to_string())?;
            let project_entry = projects_obj
                .entry(project_key.clone())
                .or_insert_with(|| serde_json::json!({}))
                .clone();
            let updated_project =
                set_servers_in_value(project_entry, "mcpServers", &private_out, false);
            projects_obj.insert(project_key, updated_project);
            write_json_file(&claude_path, &root)?;
        }

        // Don't create an empty .mcp.json in the user's repo if there is
        // nothing shared to write and the file doesn't exist yet.
        if shared_out.is_empty() && !mcp_path.exists() {
            return Ok(());
        }
        if let Some(parent) = mcp_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let updated = set_servers_in_value(mcp_root, "mcpServers", &shared_out, false);
        write_json_file(&mcp_path, &updated)
    }

    fn file_paths_watched(&self) -> Vec<PathBuf> {
        vec![Self::claude_json_path().unwrap_or_default(), self.mcp_json_path()]
    }
}
