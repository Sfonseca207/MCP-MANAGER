import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  BackupRecord,
  BootstrapSummary,
  CatalogEntry,
  CreateMcpInput,
  ReconciliationConflict,
  StatusSummary,
  UpdateMcpInput,
  WatchedProject,
} from "./types";

export const api = {
  getAppInfo: () => invoke<{ name: string; version: string }>("get_app_info"),
  bootstrapIfNeeded: () => invoke<BootstrapSummary>("bootstrap_if_needed"),
  listCatalog: () => invoke<CatalogEntry[]>("list_catalog"),
  getStatusSummary: () => invoke<StatusSummary>("get_status_summary"),
  enableMcp: (entryId: string, clients: string[]) =>
    invoke<string[]>("enable_mcp", { entryId, clients }),
  disableMcp: (entryId: string, clients: string[]) =>
    invoke<string[]>("disable_mcp", { entryId, clients }),
  createMcp: (input: CreateMcpInput) => invoke<string>("create_mcp", { input }),
  updateMcp: (input: UpdateMcpInput) => invoke<string[]>("update_mcp", { input }),
  hardDeleteMcp: (entryId: string) => invoke<void>("hard_delete_mcp", { entryId }),
  listBackups: () => invoke<BackupRecord[]>("list_backups"),
  restoreBackup: (backupId: string) => invoke<void>("restore_backup", { backupId }),
  detectConflicts: () => invoke<ReconciliationConflict[]>("detect_conflicts"),
  resolveReconciliation: (conflict: ReconciliationConflict, action: string) =>
    invoke<void>("resolve_reconciliation", { conflict, action }),
  rescanConfigs: () => invoke<BootstrapSummary>("rescan_configs"),
  addWatchedProject: (path: string, label?: string) =>
    invoke<WatchedProject>("add_watched_project", { path, label }),
  listWatchedProjects: () => invoke<WatchedProject[]>("list_watched_projects"),
  removeWatchedProject: (id: string) => invoke<void>("remove_watched_project", { id }),
  maskSecret: (value: string) => invoke<string>("mask_secret_value", { value }),
  onReconciliationConflicts: (handler: (conflicts: ReconciliationConflict[]) => void) =>
    listen<ReconciliationConflict[]>("reconciliation-conflicts", (event) => {
      handler(event.payload);
    }),
};
