use crate::adapters::ConfigAdapterRegistry;
use crate::db::Repository;
use crate::error::AppResult;
use crate::models::{ReconciliationConflict, ReconciliationKind, SourceType};
use crate::utils::{definition_hash, servers_hash};
use chrono::Utc;
use rusqlite::Connection;
use std::collections::BTreeSet;
use std::path::PathBuf;
use uuid::Uuid;

pub struct ReconciliationService;

impl ReconciliationService {
    pub fn detect_conflicts(conn: &Connection) -> AppResult<Vec<ReconciliationConflict>> {
        let watched: Vec<PathBuf> = Repository::list_watched_projects(conn)?
            .into_iter()
            .map(|p| PathBuf::from(p.path))
            .collect();

        let adapters = ConfigAdapterRegistry::all_adapters_including_projects(&watched);
        let catalog = Repository::list_catalog(conn)?;
        let mut conflicts = Vec::new();
        let now = Utc::now().to_rfc3339();

        for adapter in adapters {
            if crate::services::EnableDisableService::is_writing() {
                continue;
            }

            let target = adapter.target();
            let external = adapter.read_mcp_servers().unwrap_or_default();
            let external_hash = servers_hash(&external);
            let scope_path_str = target
                .scope_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string());

            let stored_hash = Repository::get_snapshot_hash(
                conn,
                target.client,
                target.scope_type,
                scope_path_str.as_deref(),
            )?;

            if stored_hash.as_deref() == Some(&external_hash) {
                continue;
            }

            let relevant_catalog: Vec<_> = catalog
                .iter()
                .filter(|e| {
                    e.client_origin == target.client
                        && e.scope_type == target.scope_type
                        && e.scope_path.as_deref() == scope_path_str.as_deref()
                })
                .collect();

            let catalog_names: BTreeSet<_> = relevant_catalog
                .iter()
                .map(|e| e.display_name.as_str())
                .collect();
            let external_names: BTreeSet<_> = external.keys().map(|s| s.as_str()).collect();

            for name in external_names.difference(&catalog_names) {
                if let Some(def) = external.get(*name) {
                    conflicts.push(ReconciliationConflict {
                        id: Uuid::new_v4().to_string(),
                        kind: ReconciliationKind::NewExternal,
                        client: target.client,
                        scope_type: target.scope_type,
                        scope_path: scope_path_str.clone(),
                        server_name: name.to_string(),
                        catalog_entry_id: None,
                        external_definition: Some(def.clone()),
                        detected_at: now.clone(),
                    });
                }
            }

            for entry in &relevant_catalog {
                if !external.contains_key(&entry.display_name) {
                    let enabled = entry
                        .client_states
                        .iter()
                        .any(|s| s.client == target.client && s.enabled);
                    if enabled {
                        conflicts.push(ReconciliationConflict {
                            id: Uuid::new_v4().to_string(),
                            kind: ReconciliationKind::RemovedExternal,
                            client: target.client,
                            scope_type: target.scope_type,
                            scope_path: scope_path_str.clone(),
                            server_name: entry.display_name.clone(),
                            catalog_entry_id: Some(entry.id.clone()),
                            external_definition: None,
                            detected_at: now.clone(),
                        });
                    }
                } else if let Some(ext_def) = external.get(&entry.display_name) {
                    let ext_hash = definition_hash(ext_def);
                    if ext_hash != entry.definition_hash {
                        conflicts.push(ReconciliationConflict {
                            id: Uuid::new_v4().to_string(),
                            kind: ReconciliationKind::DefinitionChanged,
                            client: target.client,
                            scope_type: target.scope_type,
                            scope_path: scope_path_str.clone(),
                            server_name: entry.display_name.clone(),
                            catalog_entry_id: Some(entry.id.clone()),
                            external_definition: Some(ext_def.clone()),
                            detected_at: now.clone(),
                        });
                    }
                }
            }

            Repository::upsert_snapshot(
                conn,
                target.client,
                target.scope_type,
                scope_path_str.as_deref(),
                &target.file_path.to_string_lossy(),
                &external_hash,
            )?;
        }

        Ok(conflicts)
    }

    pub fn import_external(
        conn: &Connection,
        conflict: &ReconciliationConflict,
    ) -> AppResult<()> {
        if let Some(def) = &conflict.external_definition {
            let hash = definition_hash(def);
            Repository::upsert_catalog_entry(
                conn,
                def,
                conflict.client,
                conflict.scope_type,
                conflict.scope_path.as_deref(),
                SourceType::AutoDetected,
                &hash,
                &[conflict.client],
            )?;
        }
        Ok(())
    }

    pub fn mark_external_removed(
        conn: &Connection,
        conflict: &ReconciliationConflict,
    ) -> AppResult<()> {
        if let Some(entry_id) = &conflict.catalog_entry_id {
            Repository::set_client_enabled(conn, entry_id, conflict.client, false)?;
        }
        Ok(())
    }
}
