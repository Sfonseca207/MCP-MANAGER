use crate::error::{AppError, AppResult};
use crate::models::{
    BackupRecord, CatalogEntry, ClientState, ClientType, CreateMcpInput, McpDefinition, ScopeType,
    SourceType, TransportType, WatchedProject,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct Repository;

impl Repository {
    pub fn is_bootstrapped(conn: &Connection) -> AppResult<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM app_meta WHERE key = 'bootstrapped'",
            [],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn set_bootstrapped(conn: &Connection) -> AppResult<()> {
        conn.execute(
            "INSERT OR REPLACE INTO app_meta (key, value) VALUES ('bootstrapped', 'true')",
            [],
        )?;
        Ok(())
    }

    pub fn upsert_catalog_entry(
        conn: &Connection,
        definition: &McpDefinition,
        client_origin: ClientType,
        scope_type: ScopeType,
        scope_path: Option<&str>,
        source: SourceType,
        definition_hash: &str,
        enabled_clients: &[ClientType],
    ) -> AppResult<String> {
        let now = Utc::now().to_rfc3339();

        if let Some(existing_id) = Self::find_catalog_id(
            conn,
            &definition.name,
            client_origin,
            scope_type,
            scope_path,
            definition_hash,
        )? {
            conn.execute(
                "UPDATE mcp_catalog SET updated_at = ?1 WHERE id = ?2",
                params![now, existing_id],
            )?;
            for client in enabled_clients {
                Self::set_client_enabled(conn, &existing_id, *client, true)?;
            }
            return Ok(existing_id);
        }

        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO mcp_catalog (
                id, display_name, transport_type, command, args, env, url, headers,
                client_origin, scope_type, scope_path, source, definition_hash, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                id,
                definition.name,
                definition.transport_type.as_str(),
                definition.command,
                json_opt(&definition.args)?,
                json_opt(&definition.env)?,
                definition.url,
                json_opt(&definition.headers)?,
                client_origin.as_str(),
                scope_type.as_str(),
                scope_path,
                source.as_str(),
                definition_hash,
                now,
                now,
            ],
        )?;

        for client in ClientType::all() {
            let enabled = enabled_clients.contains(client);
            Self::set_client_enabled(conn, &id, *client, enabled)?;
        }

        Ok(id)
    }

    pub fn find_catalog_id(
        conn: &Connection,
        display_name: &str,
        client_origin: ClientType,
        scope_type: ScopeType,
        scope_path: Option<&str>,
        definition_hash: &str,
    ) -> AppResult<Option<String>> {
        let mut stmt = conn.prepare(
            "SELECT id FROM mcp_catalog
             WHERE display_name = ?1 AND client_origin = ?2 AND scope_type = ?3
             AND COALESCE(scope_path, '') = COALESCE(?4, '') AND definition_hash = ?5
             LIMIT 1",
        )?;
        let mut rows = stmt.query(params![
            display_name,
            client_origin.as_str(),
            scope_type.as_str(),
            scope_path,
            definition_hash
        ])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_client_enabled(
        conn: &Connection,
        mcp_id: &str,
        client: ClientType,
        enabled: bool,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM mcp_client_state WHERE mcp_id = ?1 AND client = ?2",
                params![mcp_id, client.as_str()],
                |row| row.get(0),
            )
            .ok();

        if let Some(state_id) = existing {
            if enabled {
                conn.execute(
                    "UPDATE mcp_client_state SET enabled = 1, last_enabled_at = ?1 WHERE id = ?2",
                    params![now, state_id],
                )?;
            } else {
                conn.execute(
                    "UPDATE mcp_client_state SET enabled = 0, last_disabled_at = ?1 WHERE id = ?2",
                    params![now, state_id],
                )?;
            }
        } else {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO mcp_client_state (id, mcp_id, client, enabled, last_enabled_at, last_disabled_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    mcp_id,
                    client.as_str(),
                    enabled as i32,
                    if enabled { Some(now.clone()) } else { None::<String> },
                    if enabled { None::<String> } else { Some(now) },
                ],
            )?;
        }
        Ok(())
    }

    pub fn list_catalog(conn: &Connection) -> AppResult<Vec<CatalogEntry>> {
        let mut stmt = conn.prepare(
            "SELECT id, display_name, transport_type, command, args, env, url, headers,
                    client_origin, scope_type, scope_path, source, definition_hash, created_at, updated_at
             FROM mcp_catalog ORDER BY display_name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, String>(12)?,
                row.get::<_, String>(13)?,
                row.get::<_, String>(14)?,
            ))
        })?;

        let mut entries = Vec::new();
        for row in rows {
            let (
                id,
                display_name,
                transport_type,
                command,
                args,
                env,
                url,
                headers,
                client_origin,
                scope_type,
                scope_path,
                source,
                definition_hash,
                created_at,
                updated_at,
            ) = row?;
            entries.push(CatalogEntry {
                id: id.clone(),
                display_name,
                transport_type: TransportType::from_str(&transport_type)
                    .unwrap_or(TransportType::Stdio),
                command,
                args: parse_json_opt(args)?,
                env: parse_json_opt(env)?,
                url,
                headers: parse_json_opt(headers)?,
                client_origin: ClientType::from_str(&client_origin)
                    .unwrap_or(ClientType::Cursor),
                scope_type: ScopeType::from_str(&scope_type).unwrap_or(ScopeType::Global),
                scope_path,
                source: SourceType::from_str(&source).unwrap_or(SourceType::Manual),
                definition_hash,
                created_at,
                updated_at,
                client_states: Self::get_client_states(conn, &id)?,
            });
        }
        Ok(entries)
    }

    pub fn get_catalog_entry(conn: &Connection, id: &str) -> AppResult<CatalogEntry> {
        Self::list_catalog(conn)?
            .into_iter()
            .find(|e| e.id == id)
            .ok_or_else(|| AppError::Message(format!("Catalog entry {id} not found")))
    }

    fn get_client_states(conn: &Connection, mcp_id: &str) -> AppResult<Vec<ClientState>> {
        let mut stmt = conn.prepare(
            "SELECT client, enabled, last_enabled_at, last_disabled_at
             FROM mcp_client_state WHERE mcp_id = ?1",
        )?;
        let rows = stmt.query_map(params![mcp_id], |row| {
            Ok(ClientState {
                client: ClientType::from_str(&row.get::<_, String>(0)?)
                    .unwrap_or(ClientType::Cursor),
                enabled: row.get::<_, i32>(1)? != 0,
                last_enabled_at: row.get(2)?,
                last_disabled_at: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn create_manual_entry(conn: &Connection, input: &CreateMcpInput) -> AppResult<String> {
        let definition = McpDefinition {
            name: input.display_name.clone(),
            transport_type: input.transport_type,
            command: input.command.clone(),
            args: input.args.clone(),
            env: input.env.clone(),
            url: input.url.clone(),
            headers: input.headers.clone(),
        };
        let hash = crate::utils::definition_hash(&definition);
        Self::upsert_catalog_entry(
            conn,
            &definition,
            input.client_origin,
            input.scope_type,
            input.scope_path.as_deref(),
            SourceType::Manual,
            &hash,
            &[],
        )
    }

    pub fn update_catalog_entry(conn: &Connection, input: &crate::models::UpdateMcpInput) -> AppResult<()> {
        let definition = McpDefinition {
            name: input.display_name.clone(),
            transport_type: input.transport_type,
            command: input.command.clone(),
            args: input.args.clone(),
            env: input.env.clone(),
            url: input.url.clone(),
            headers: input.headers.clone(),
        };
        let hash = crate::utils::definition_hash(&definition);
        let now = Utc::now().to_rfc3339();

        let rows = conn.execute(
            "UPDATE mcp_catalog SET
                display_name = ?1, transport_type = ?2, command = ?3, args = ?4,
                env = ?5, url = ?6, headers = ?7, definition_hash = ?8, updated_at = ?9
             WHERE id = ?10",
            params![
                input.display_name,
                input.transport_type.as_str(),
                input.command,
                json_opt(&input.args)?,
                json_opt(&input.env)?,
                input.url,
                json_opt(&input.headers)?,
                hash,
                now,
                input.id,
            ],
        )?;

        if rows == 0 {
            return Err(AppError::Message(format!("Catalog entry {} not found", input.id)));
        }
        Ok(())
    }

    pub fn hard_delete(conn: &Connection, id: &str) -> AppResult<()> {
        conn.execute("DELETE FROM mcp_catalog WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn insert_backup(
        conn: &Connection,
        client: ClientType,
        scope_type: ScopeType,
        scope_path: Option<&str>,
        file_path: &str,
        backup_path: &str,
        reason: &str,
    ) -> AppResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO mcp_backups (id, client, scope_type, scope_path, file_path, backup_path, created_at, reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                client.as_str(),
                scope_type.as_str(),
                scope_path,
                file_path,
                backup_path,
                now,
                reason
            ],
        )?;
        Ok(id)
    }

    pub fn list_backups(conn: &Connection) -> AppResult<Vec<BackupRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, client, scope_type, scope_path, file_path, backup_path, created_at, reason
             FROM mcp_backups ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BackupRecord {
                id: row.get(0)?,
                client: ClientType::from_str(&row.get::<_, String>(1)?)
                    .unwrap_or(ClientType::Cursor),
                scope_type: ScopeType::from_str(&row.get::<_, String>(2)?)
                    .unwrap_or(ScopeType::Global),
                scope_path: row.get(3)?,
                file_path: row.get(4)?,
                backup_path: row.get(5)?,
                created_at: row.get(6)?,
                reason: row.get(7)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Deletes DB rows beyond the `keep` newest backups for a target file and
    /// returns the backup file paths that should be removed from disk.
    pub fn prune_backups(conn: &Connection, file_path: &str, keep: usize) -> AppResult<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT id, backup_path FROM mcp_backups WHERE file_path = ?1 ORDER BY created_at DESC",
        )?;
        let rows: Vec<(String, String)> = stmt
            .query_map(params![file_path], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        let mut removed = Vec::new();
        for (id, backup_path) in rows.into_iter().skip(keep) {
            conn.execute("DELETE FROM mcp_backups WHERE id = ?1", params![id])?;
            removed.push(backup_path);
        }
        Ok(removed)
    }

    pub fn upsert_snapshot(
        conn: &Connection,
        client: ClientType,
        scope_type: ScopeType,
        scope_path: Option<&str>,
        file_path: &str,
        servers_hash: &str,
    ) -> AppResult<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO config_snapshots (id, client, scope_type, scope_path, file_path, servers_hash, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(client, scope_type, scope_path) DO UPDATE SET
                servers_hash = excluded.servers_hash,
                file_path = excluded.file_path,
                updated_at = excluded.updated_at",
            params![
                id,
                client.as_str(),
                scope_type.as_str(),
                scope_path,
                file_path,
                servers_hash,
                now
            ],
        )?;
        Ok(())
    }

    pub fn get_snapshot_hash(
        conn: &Connection,
        client: ClientType,
        scope_type: ScopeType,
        scope_path: Option<&str>,
    ) -> AppResult<Option<String>> {
        let result = conn.query_row(
            "SELECT servers_hash FROM config_snapshots
             WHERE client = ?1 AND scope_type = ?2 AND COALESCE(scope_path, '') = COALESCE(?3, '')",
            params![client.as_str(), scope_type.as_str(), scope_path],
            |row| row.get(0),
        );
        match result {
            Ok(hash) => Ok(Some(hash)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn add_watched_project(conn: &Connection, path: &str, label: Option<&str>) -> AppResult<WatchedProject> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR IGNORE INTO watched_projects (id, path, label, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, path, label, now],
        )?;
        Ok(Self::list_watched_projects(conn)?
            .into_iter()
            .find(|p| p.path == path)
            .ok_or_else(|| AppError::Message("Failed to add watched project".into()))?)
    }

    pub fn list_watched_projects(conn: &Connection) -> AppResult<Vec<WatchedProject>> {
        let mut stmt = conn.prepare(
            "SELECT id, path, label, created_at FROM watched_projects ORDER BY path",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(WatchedProject {
                id: row.get(0)?,
                path: row.get(1)?,
                label: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn remove_watched_project(conn: &Connection, id: &str) -> AppResult<()> {
        conn.execute("DELETE FROM watched_projects WHERE id = ?1", params![id])?;
        Ok(())
    }
}

fn json_opt<T: serde::Serialize>(value: &Option<T>) -> AppResult<Option<String>> {
    match value {
        Some(v) => Ok(Some(serde_json::to_string(v)?)),
        None => Ok(None),
    }
}

fn parse_json_opt<T: for<'de> serde::Deserialize<'de>>(value: Option<String>) -> AppResult<Option<T>> {
    match value {
        Some(v) => Ok(Some(serde_json::from_str(&v)?)),
        None => Ok(None),
    }
}
