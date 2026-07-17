use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS mcp_catalog (
            id TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            transport_type TEXT NOT NULL,
            command TEXT,
            args TEXT,
            env TEXT,
            url TEXT,
            headers TEXT,
            client_origin TEXT NOT NULL,
            scope_type TEXT NOT NULL DEFAULT 'global',
            scope_path TEXT,
            source TEXT NOT NULL,
            definition_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS mcp_client_state (
            id TEXT PRIMARY KEY,
            mcp_id TEXT NOT NULL REFERENCES mcp_catalog(id) ON DELETE CASCADE,
            client TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 0,
            last_enabled_at TEXT,
            last_disabled_at TEXT,
            UNIQUE(mcp_id, client)
        );

        CREATE TABLE IF NOT EXISTS mcp_backups (
            id TEXT PRIMARY KEY,
            client TEXT NOT NULL,
            scope_type TEXT NOT NULL DEFAULT 'global',
            scope_path TEXT,
            file_path TEXT NOT NULL,
            backup_path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            reason TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS config_snapshots (
            id TEXT PRIMARY KEY,
            client TEXT NOT NULL,
            scope_type TEXT NOT NULL,
            scope_path TEXT,
            file_path TEXT NOT NULL,
            servers_hash TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(client, scope_type, scope_path)
        );

        CREATE TABLE IF NOT EXISTS watched_projects (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            label TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_catalog_name ON mcp_catalog(display_name);
        CREATE INDEX IF NOT EXISTS idx_client_state_mcp ON mcp_client_state(mcp_id);
        ",
    )?;

    let version: i64 = conn
        .query_row(
            "SELECT COALESCE((SELECT version FROM schema_version LIMIT 1), 0)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version == 0 {
        conn.execute("INSERT OR REPLACE INTO schema_version (version) VALUES (1)", [])?;
    }

    Ok(())
}
