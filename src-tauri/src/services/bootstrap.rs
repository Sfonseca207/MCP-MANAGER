use crate::adapters::ConfigAdapterRegistry;
use crate::db::Repository;
use crate::error::AppResult;
use crate::models::{BootstrapSummary, ClientType, SourceType};
use crate::utils::{definition_hash, servers_hash};
use rusqlite::Connection;

pub struct BootstrapService;

impl BootstrapService {
    pub fn bootstrap_if_needed(conn: &Connection) -> AppResult<BootstrapSummary> {
        let first_run = !Repository::is_bootstrapped(conn)?;
        let mut counts = BootstrapSummary {
            claude_code_count: 0,
            cursor_count: 0,
            claude_desktop_count: 0,
            vscode_count: 0,
            windsurf_count: 0,
            total: 0,
            first_run,
        };

        for adapter in ConfigAdapterRegistry::all_global_adapters() {
            let target = adapter.target();
            let servers = adapter.read_mcp_servers().unwrap_or_default();
            let count = servers.len() as u32;

            match target.client {
                ClientType::ClaudeCode => counts.claude_code_count = count,
                ClientType::Cursor => counts.cursor_count = count,
                ClientType::ClaudeDesktop => counts.claude_desktop_count = count,
                ClientType::VsCode => counts.vscode_count = count,
                ClientType::Windsurf => counts.windsurf_count = count,
            }

            let hash = servers_hash(&servers);
            Repository::upsert_snapshot(
                conn,
                target.client,
                target.scope_type,
                target.scope_path.as_ref().map(|p| p.to_string_lossy().to_string()).as_deref(),
                &target.file_path.to_string_lossy(),
                &hash,
            )?;

            for (_, definition) in &servers {
                let def_hash = definition_hash(definition);
                Repository::upsert_catalog_entry(
                    conn,
                    definition,
                    target.client,
                    target.scope_type,
                    target.scope_path.as_ref().map(|p| p.to_string_lossy().to_string()).as_deref(),
                    SourceType::AutoDetected,
                    &def_hash,
                    &[target.client],
                )?;
            }
        }

        counts.total = counts.claude_code_count
            + counts.cursor_count
            + counts.claude_desktop_count
            + counts.vscode_count
            + counts.windsurf_count;

        if first_run {
            Repository::set_bootstrapped(conn)?;
        }

        Ok(counts)
    }

    pub fn rescan_all(conn: &Connection) -> AppResult<BootstrapSummary> {
        Self::bootstrap_if_needed(conn)
    }
}
