use crate::adapters::ConfigAdapterRegistry;
use crate::db::Repository;
use crate::error::AppResult;
use crate::models::{CatalogEntry, ClientType, UpdateMcpInput};
use crate::services::BackupService;
use crate::utils::servers_hash;
use rusqlite::Connection;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static WRITING_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

pub struct EnableDisableService;

impl EnableDisableService {
    pub fn is_writing() -> bool {
        WRITING_IN_PROGRESS.load(Ordering::SeqCst)
    }

    pub fn update_entry(conn: &Connection, input: &UpdateMcpInput) -> AppResult<Vec<String>> {
        let old_entry = Repository::get_catalog_entry(conn, &input.id)?;
        Repository::update_catalog_entry(conn, input)?;
        let new_entry = Repository::get_catalog_entry(conn, &input.id)?;

        let mut restart_messages = Vec::new();
        for state in &new_entry.client_states {
            if !state.enabled {
                continue;
            }
            Self::sync_entry_on_client(conn, &old_entry, &new_entry, state.client)?;
            restart_messages.push(format!(
                "Reinicia {} para aplicar cambios.",
                state.client.label()
            ));
        }

        if restart_messages.is_empty() {
            restart_messages.push("MCP actualizado en el catálogo.".to_string());
        }

        Ok(restart_messages)
    }

    fn sync_entry_on_client(
        conn: &Connection,
        old_entry: &CatalogEntry,
        new_entry: &CatalogEntry,
        client: ClientType,
    ) -> AppResult<()> {
        let adapter = ConfigAdapterRegistry::adapter_for(
            client,
            new_entry.scope_type,
            new_entry.scope_path.as_deref().map(Path::new),
        )?;

        WRITING_IN_PROGRESS.store(true, Ordering::SeqCst);
        let result = (|| {
            BackupService::create_backup(conn, adapter.as_ref(), "pre_edit")?;

            let mut servers = adapter.read_mcp_servers()?;
            servers.remove(&old_entry.display_name);
            servers.insert(
                new_entry.display_name.clone(),
                new_entry.to_definition(),
            );
            adapter.write_mcp_servers(&servers)?;

            let target = adapter.target();
            Repository::upsert_snapshot(
                conn,
                target.client,
                target.scope_type,
                target
                    .scope_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .as_deref(),
                &target.file_path.to_string_lossy(),
                &servers_hash(&servers),
            )?;
            Ok(())
        })();
        WRITING_IN_PROGRESS.store(false, Ordering::SeqCst);
        result
    }

    pub fn enable(
        conn: &Connection,
        entry_id: &str,
        clients: &[ClientType],
    ) -> AppResult<Vec<String>> {
        let entry = Repository::get_catalog_entry(conn, entry_id)?;
        let mut restart_messages = Vec::new();

        for client in clients {
            let adapter = ConfigAdapterRegistry::adapter_for(
                *client,
                entry.scope_type,
                entry.scope_path.as_deref().map(Path::new),
            )?;
            Self::enable_on_adapter(conn, &entry, adapter.as_ref())?;
            restart_messages.push(format!("Reinicia {} para aplicar cambios.", client.label()));
        }

        Ok(restart_messages)
    }

    pub fn disable(
        conn: &Connection,
        entry_id: &str,
        clients: &[ClientType],
    ) -> AppResult<Vec<String>> {
        let entry = Repository::get_catalog_entry(conn, entry_id)?;
        let mut restart_messages = Vec::new();

        for client in clients {
            let adapter = ConfigAdapterRegistry::adapter_for(
                *client,
                entry.scope_type,
                entry.scope_path.as_deref().map(Path::new),
            )?;
            Self::disable_on_adapter(conn, &entry, adapter.as_ref(), *client)?;
            restart_messages.push(format!("Reinicia {} para aplicar cambios.", client.label()));
        }

        Ok(restart_messages)
    }

    fn enable_on_adapter(
        conn: &Connection,
        entry: &CatalogEntry,
        adapter: &dyn crate::adapters::ConfigAdapter,
    ) -> AppResult<()> {
        WRITING_IN_PROGRESS.store(true, Ordering::SeqCst);
        let result = (|| {
            BackupService::create_backup(conn, adapter, "pre_enable")?;

            let mut servers = adapter.read_mcp_servers()?;
            let definition = entry.to_definition();
            servers.insert(definition.name.clone(), definition);
            adapter.write_mcp_servers(&servers)?;

            let target = adapter.target();
            Repository::upsert_snapshot(
                conn,
                target.client,
                target.scope_type,
                target
                    .scope_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .as_deref(),
                &target.file_path.to_string_lossy(),
                &servers_hash(&servers),
            )?;

            Repository::set_client_enabled(conn, &entry.id, target.client, true)?;
            Ok(())
        })();
        WRITING_IN_PROGRESS.store(false, Ordering::SeqCst);
        result
    }

    fn disable_on_adapter(
        conn: &Connection,
        entry: &CatalogEntry,
        adapter: &dyn crate::adapters::ConfigAdapter,
        client: ClientType,
    ) -> AppResult<()> {
        WRITING_IN_PROGRESS.store(true, Ordering::SeqCst);
        let result = (|| {
            BackupService::create_backup(conn, adapter, "pre_disable")?;

            let mut servers = adapter.read_mcp_servers()?;
            servers.remove(&entry.display_name);
            adapter.write_mcp_servers(&servers)?;

            let target = adapter.target();
            Repository::upsert_snapshot(
                conn,
                target.client,
                target.scope_type,
                target
                    .scope_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .as_deref(),
                &target.file_path.to_string_lossy(),
                &servers_hash(&servers),
            )?;

            Repository::set_client_enabled(conn, &entry.id, client, false)?;
            Ok(())
        })();
        WRITING_IN_PROGRESS.store(false, Ordering::SeqCst);
        result
    }

    pub fn sync_enabled_state_from_files(conn: &Connection) -> AppResult<()> {
        let entries = Repository::list_catalog(conn)?;
        for entry in entries {
            for client in ClientType::all() {
                if let Ok(adapter) = ConfigAdapterRegistry::adapter_for(
                    *client,
                    entry.scope_type,
                    entry.scope_path.as_deref().map(Path::new),
                ) {
                    let servers = adapter.read_mcp_servers().unwrap_or_default();
                    let enabled = servers.contains_key(&entry.display_name);
                    Repository::set_client_enabled(conn, &entry.id, *client, enabled)?;
                }
            }
        }
        Ok(())
    }
}
