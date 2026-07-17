export type ClientType =
  | "claude_code"
  | "cursor"
  | "claude_desktop"
  | "vscode"
  | "windsurf";

export type ScopeType = "global" | "project";
export type TransportType = "stdio" | "http" | "sse";
export type SourceType = "auto_detectado" | "manual";

export type ReconciliationKind =
  | "new_external"
  | "removed_external"
  | "definition_changed"
  | "name_collision";

export interface ClientState {
  client: ClientType;
  enabled: boolean;
  last_enabled_at?: string | null;
  last_disabled_at?: string | null;
}

export interface CatalogEntry {
  id: string;
  display_name: string;
  transport_type: TransportType;
  command?: string | null;
  args?: string[] | null;
  env?: Record<string, string> | null;
  url?: string | null;
  headers?: Record<string, string> | null;
  client_origin: ClientType;
  scope_type: ScopeType;
  scope_path?: string | null;
  source: SourceType;
  definition_hash: string;
  created_at: string;
  updated_at: string;
  client_states: ClientState[];
}

export interface BootstrapSummary {
  claude_code_count: number;
  cursor_count: number;
  claude_desktop_count: number;
  vscode_count: number;
  windsurf_count: number;
  total: number;
  first_run: boolean;
}

export interface StatusSummary {
  total_catalog: number;
  active_by_client: Record<string, number>;
  estimated_tools_by_client: Record<string, number>;
  cursor_tool_limit: number;
  cursor_over_limit: boolean;
}

export interface BackupRecord {
  id: string;
  client: ClientType;
  scope_type: ScopeType;
  scope_path?: string | null;
  file_path: string;
  backup_path: string;
  created_at: string;
  reason: string;
}

export interface ReconciliationConflict {
  id: string;
  kind: ReconciliationKind;
  client: ClientType;
  scope_type: ScopeType;
  scope_path?: string | null;
  server_name: string;
  catalog_entry_id?: string | null;
  external_definition?: {
    name: string;
    transport_type: TransportType;
    command?: string | null;
    args?: string[] | null;
    env?: Record<string, string> | null;
    url?: string | null;
    headers?: Record<string, string> | null;
  } | null;
  detected_at: string;
}

export interface CreateMcpInput {
  display_name: string;
  transport_type: TransportType;
  command?: string | null;
  args?: string[] | null;
  env?: Record<string, string> | null;
  url?: string | null;
  headers?: Record<string, string> | null;
  client_origin: ClientType;
  scope_type: ScopeType;
  scope_path?: string | null;
}

export interface UpdateMcpInput {
  id: string;
  display_name: string;
  transport_type: TransportType;
  command?: string | null;
  args?: string[] | null;
  env?: Record<string, string> | null;
  url?: string | null;
  headers?: Record<string, string> | null;
}

export interface WatchedProject {
  id: string;
  path: string;
  label?: string | null;
  created_at: string;
}

export const CLIENT_LABELS: Record<ClientType, string> = {
  claude_code: "Claude Code",
  cursor: "Cursor",
  claude_desktop: "Claude Desktop",
  vscode: "VS Code",
  windsurf: "Windsurf",
};

export const ALL_CLIENTS: ClientType[] = [
  "claude_code",
  "cursor",
  "claude_desktop",
  "vscode",
  "windsurf",
];

/** MCPs cuyo origen pertenece a este entorno/IDE */
export function filterEntriesByClient(
  entries: CatalogEntry[],
  client: ClientType,
): CatalogEntry[] {
  return entries.filter((e) => e.client_origin === client);
}

export const CLIENT_SHORT: Record<ClientType, string> = {
  claude_code: "Claude Code",
  cursor: "Cursor",
  claude_desktop: "Claude Desktop",
  vscode: "VS Code",
  windsurf: "Windsurf",
};
