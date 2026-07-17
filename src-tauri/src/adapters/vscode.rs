use super::{
    extract_servers_from_value, read_json_file, set_servers_in_value, write_json_file, ConfigAdapter,
    ConfigTarget,
};
use crate::error::AppResult;
use crate::models::{McpDefinition, ScopeType};
use crate::utils::home_dir;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct VsCodeUserAdapter {
    config_path: PathBuf,
}

impl VsCodeUserAdapter {
    pub fn new() -> Self {
        Self {
            config_path: Self::resolve_user_config_path(),
        }
    }

    fn resolve_user_config_path() -> PathBuf {
        let base = home_dir()
            .unwrap_or_default()
            .join("Library")
            .join("Application Support")
            .join("Code")
            .join("User");

        // Try common profile locations
        let candidates = [
            base.join("mcp.json"),
            base.join("globalStorage").join("mcp.json"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return candidate.clone();
            }
        }

        base.join("mcp.json")
    }
}

impl ConfigAdapter for VsCodeUserAdapter {
    fn target(&self) -> ConfigTarget {
        ConfigTarget {
            client: crate::models::ClientType::VsCode,
            scope_type: ScopeType::Global,
            scope_path: None,
            file_path: self.config_path.clone(),
            servers_key: "servers".into(),
            use_windsurf_format: false,
        }
    }

    fn read_mcp_servers(&self) -> AppResult<BTreeMap<String, McpDefinition>> {
        let root = read_json_file(&self.config_path)?;
        extract_servers_from_value(&root, "servers")
    }

    fn write_mcp_servers(&self, servers: &BTreeMap<String, McpDefinition>) -> AppResult<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let root = read_json_file(&self.config_path)?;
        let updated = set_servers_in_value(root, "servers", servers, false);
        write_json_file(&self.config_path, &updated)
    }
}

pub struct VsCodeWorkspaceAdapter {
    project_path: PathBuf,
}

impl VsCodeWorkspaceAdapter {
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
        }
    }

    fn config_path(&self) -> PathBuf {
        self.project_path.join(".vscode").join("mcp.json")
    }
}

impl ConfigAdapter for VsCodeWorkspaceAdapter {
    fn target(&self) -> ConfigTarget {
        ConfigTarget {
            client: crate::models::ClientType::VsCode,
            scope_type: ScopeType::Project,
            scope_path: Some(self.project_path.clone()),
            file_path: self.config_path(),
            servers_key: "servers".into(),
            use_windsurf_format: false,
        }
    }

    fn read_mcp_servers(&self) -> AppResult<BTreeMap<String, McpDefinition>> {
        let root = read_json_file(&self.config_path())?;
        extract_servers_from_value(&root, "servers")
    }

    fn write_mcp_servers(&self, servers: &BTreeMap<String, McpDefinition>) -> AppResult<()> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let root = read_json_file(&path)?;
        let updated = set_servers_in_value(root, "servers", servers, false);
        write_json_file(&path, &updated)
    }
}
