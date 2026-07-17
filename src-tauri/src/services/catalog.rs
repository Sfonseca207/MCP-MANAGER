use crate::db::Repository;
use crate::error::AppResult;
use crate::models::{CatalogEntry, ClientType, StatusSummary, UpdateMcpInput};
use crate::services::EnableDisableService;
use rusqlite::Connection;
use std::collections::BTreeMap;

pub struct CatalogService;

impl CatalogService {
    pub fn list(conn: &Connection) -> AppResult<Vec<CatalogEntry>> {
        Repository::list_catalog(conn)
    }

    pub fn update(conn: &Connection, input: &UpdateMcpInput) -> AppResult<Vec<String>> {
        EnableDisableService::update_entry(conn, input)
    }

    pub fn status_summary(conn: &Connection) -> AppResult<StatusSummary> {
        let entries = Repository::list_catalog(conn)?;
        let mut active_by_client: BTreeMap<String, u32> = BTreeMap::new();
        let mut estimated_tools_by_client: BTreeMap<String, u32> = BTreeMap::new();

        for client in ClientType::all() {
            active_by_client.insert(client.as_str().to_string(), 0);
            estimated_tools_by_client.insert(client.as_str().to_string(), 0);
        }

        for entry in &entries {
            let def = entry.to_definition();
            for state in &entry.client_states {
                if state.enabled {
                    let key = state.client.as_str().to_string();
                    *active_by_client.entry(key.clone()).or_insert(0) += 1;
                    *estimated_tools_by_client.entry(key).or_insert(0) +=
                        def.estimated_tool_count();
                }
            }
        }

        let cursor_tools = estimated_tools_by_client
            .get(ClientType::Cursor.as_str())
            .copied()
            .unwrap_or(0);

        Ok(StatusSummary {
            total_catalog: entries.len() as u32,
            active_by_client,
            estimated_tools_by_client,
            cursor_tool_limit: 40,
            cursor_over_limit: cursor_tools > 40,
        })
    }
}
