import { useCallback, useEffect, useMemo, useState } from "react";
import { ask } from "@tauri-apps/plugin-dialog";
import { McpForm } from "./components/McpForm";
import { BackupsView } from "./components/BackupsView";
import { CatalogList } from "./components/CatalogList";
import { CatalogToolbar } from "./components/CatalogToolbar";
import { OnboardingBanner } from "./components/OnboardingBanner";
import { ReconciliationModal } from "./components/ReconciliationModal";
import { ClientEnvironmentView } from "./components/ClientEnvironmentView";
import { SettingsView } from "./components/SettingsView";
import { Sidebar, clientFromPage, isClientPage, type Page } from "./components/Sidebar";
import { StatusPanel } from "./components/StatusPanel";
import { Toast } from "./components/Toast";
import { api } from "./lib/tauri-api";
import { filterCatalogEntries, type StatusFilter } from "./lib/filter-catalog";
import type {
  BackupRecord,
  BootstrapSummary,
  CatalogEntry,
  ClientType,
  CreateMcpInput,
  ReconciliationConflict,
  StatusSummary,
  UpdateMcpInput,
  WatchedProject,
} from "./lib/types";

function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [catalog, setCatalog] = useState<CatalogEntry[]>([]);
  const [summary, setSummary] = useState<StatusSummary | null>(null);
  const [bootstrap, setBootstrap] = useState<BootstrapSummary | null>(null);
  const [backups, setBackups] = useState<BackupRecord[]>([]);
  const [projects, setProjects] = useState<WatchedProject[]>([]);
  const [conflicts, setConflicts] = useState<ReconciliationConflict[]>([]);
  const [showMcpForm, setShowMcpForm] = useState(false);
  const [mcpFormMode, setMcpFormMode] = useState<"create" | "edit">("create");
  const [editingEntry, setEditingEntry] = useState<CatalogEntry | null>(null);
  const [addFormClient, setAddFormClient] = useState<ClientType | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [scopeFilter, setScopeFilter] = useState<"all" | "global" | "project">("all");
  const [catalogQuery, setCatalogQuery] = useState("");
  const [catalogStatusFilter, setCatalogStatusFilter] = useState<StatusFilter>("all");
  const [loading, setLoading] = useState(true);

  const catalogFilteredCount = useMemo(
    () =>
      filterCatalogEntries(catalog, {
        query: catalogQuery,
        scope: scopeFilter,
        status: catalogStatusFilter,
      }).length,
    [catalog, catalogQuery, scopeFilter, catalogStatusFilter],
  );

  const refresh = useCallback(async () => {
    const [catalogData, summaryData, backupsData, projectsData] = await Promise.all([
      api.listCatalog(),
      api.getStatusSummary(),
      api.listBackups(),
      api.listWatchedProjects(),
    ]);
    setCatalog(catalogData);
    setSummary(summaryData);
    setBackups(backupsData);
    setProjects(projectsData);
  }, []);

  useEffect(() => {
    (async () => {
      try {
        const boot = await api.bootstrapIfNeeded();
        setBootstrap(boot);
        await refresh();
        const detected = await api.detectConflicts();
        setConflicts(detected);
      } finally {
        setLoading(false);
      }
    })();
  }, [refresh]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    api.onReconciliationConflicts((newConflicts) => {
      setConflicts(newConflicts);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  const handleToggle = async (
    entry: CatalogEntry,
    client: ClientType,
    enable: boolean,
  ) => {
    const messages = enable
      ? await api.enableMcp(entry.id, [client])
      : await api.disableMcp(entry.id, [client]);
    await refresh();
    setToast(messages.join(" "));
  };

  const handleDelete = async (entry: CatalogEntry) => {
    const confirmed = await ask(
      `¿Eliminar "${entry.display_name}" del catálogo permanentemente? Esta acción no se puede deshacer.`,
      { title: "Eliminar MCP", kind: "warning" },
    );
    if (!confirmed) return;

    const doubleConfirm = await ask(
      "Confirma de nuevo: el registro se borrará para siempre del catálogo local.",
      { title: "Confirmación final", kind: "error" },
    );
    if (!doubleConfirm) return;

    await api.hardDeleteMcp(entry.id);
    await refresh();
    setToast("MCP eliminado del catálogo.");
  };

  const handleUpdate = async (input: UpdateMcpInput) => {
    const messages = await api.updateMcp(input);
    await refresh();
    setToast(
      messages.length
        ? messages.join(" ")
        : "MCP actualizado en el catálogo.",
    );
  };

  const handleCreate = async (input: CreateMcpInput) => {
    await api.createMcp(input);
    await refresh();
    setToast("MCP agregado al catálogo (deshabilitado).");
  };

  const handleRestore = async (id: string) => {
    const confirmed = await ask("¿Restaurar este backup?", {
      title: "Restaurar backup",
      kind: "warning",
    });
    if (!confirmed) return;
    await api.restoreBackup(id);
    await refresh();
    setToast("Backup restaurado. Reinicia los clientes afectados.");
  };

  const handleResolve = async (
    conflict: ReconciliationConflict,
    action: "import" | "mark_disabled" | "ignore",
  ) => {
    await api.resolveReconciliation(conflict, action);
    setConflicts((prev) => prev.filter((c) => c.id !== conflict.id));
    await refresh();
    setToast("Conflicto resuelto.");
  };

  const handleRescan = async () => {
    const result = await api.rescanConfigs();
    setBootstrap(result);
    await refresh();
    const detected = await api.detectConflicts();
    setConflicts(detected);
    setToast("Configuraciones re-escaneadas.");
  };

  const activeClient = clientFromPage(page);

  const openAddForm = (client?: ClientType) => {
    setMcpFormMode("create");
    setEditingEntry(null);
    setAddFormClient(client ?? null);
    setShowMcpForm(true);
  };

  const openEditForm = (entry: CatalogEntry) => {
    setMcpFormMode("edit");
    setEditingEntry(entry);
    setAddFormClient(null);
    setShowMcpForm(true);
  };

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center text-[var(--text-muted)]">
        Cargando MCP Manager...
      </div>
    );
  }

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar
        current={page}
        onNavigate={setPage}
        onAdd={() => openAddForm(activeClient ?? undefined)}
        onRefresh={handleRescan}
        activeCounts={summary?.active_by_client}
      />

      <main className="flex-1 overflow-y-auto p-6">
        <OnboardingBanner summary={bootstrap} />

        {page === "dashboard" && (
          <div className="space-y-6">
            <h1 className="text-2xl font-semibold">Dashboard</h1>
            <StatusPanel summary={summary} />
            <div>
              <h2 className="mb-3 text-lg font-medium">MCPs recientes</h2>
              <CatalogList
                entries={catalog.slice(0, 5)}
                onToggle={handleToggle}
                onEdit={openEditForm}
                onDelete={handleDelete}
              />
            </div>
          </div>
        )}

        {page === "catalog" && (
          <div className="space-y-4">
            <h1 className="text-2xl font-semibold">Catálogo</h1>

            <CatalogToolbar
              query={catalogQuery}
              onQueryChange={setCatalogQuery}
              statusFilter={catalogStatusFilter}
              onStatusFilterChange={setCatalogStatusFilter}
              scopeFilter={scopeFilter}
              onScopeFilterChange={setScopeFilter}
              totalCount={catalog.length}
              filteredCount={catalogFilteredCount}
            />

            <CatalogList
              entries={catalog}
              onToggle={handleToggle}
              onEdit={openEditForm}
              onDelete={handleDelete}
              filterScope={scopeFilter}
              query={catalogQuery}
              statusFilter={catalogStatusFilter}
            />
          </div>
        )}

        {isClientPage(page) && activeClient && (
          <ClientEnvironmentView
            client={activeClient}
            entries={catalog}
            summary={summary}
            onToggle={handleToggle}
            onEdit={openEditForm}
            onDelete={handleDelete}
          />
        )}

        {page === "backups" && (
          <div className="space-y-4">
            <h1 className="text-2xl font-semibold">Backups</h1>
            <BackupsView backups={backups} onRestore={handleRestore} />
          </div>
        )}

        {page === "settings" && (
          <div className="space-y-4">
            <h1 className="text-2xl font-semibold">Ajustes</h1>
            <SettingsView
              projects={projects}
              onAddProject={async (path) => {
                await api.addWatchedProject(path);
                await refresh();
              }}
              onRemoveProject={async (id) => {
                await api.removeWatchedProject(id);
                await refresh();
              }}
            />
          </div>
        )}
      </main>

      <McpForm
        open={showMcpForm}
        mode={mcpFormMode}
        entry={editingEntry}
        onClose={() => {
          setShowMcpForm(false);
          setEditingEntry(null);
          setAddFormClient(null);
        }}
        defaultClient={addFormClient}
        onCreate={handleCreate}
        onUpdate={handleUpdate}
      />

      <ReconciliationModal
        conflicts={conflicts}
        onResolve={handleResolve}
        onClose={() => setConflicts([])}
      />

      <Toast message={toast} onClose={() => setToast(null)} />
    </div>
  );
}

export default App;
