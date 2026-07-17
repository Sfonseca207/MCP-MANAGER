import { useMemo, useState } from "react";
import type { CatalogEntry, ClientType, StatusSummary } from "../lib/types";
import { CLIENT_LABELS, filterEntriesByClient } from "../lib/types";
import { filterCatalogEntries, type StatusFilter } from "../lib/filter-catalog";
import { CatalogList } from "./CatalogList";
import { CatalogToolbar } from "./CatalogToolbar";
import { ClientStatusPanel } from "./ClientStatusPanel";

interface ClientEnvironmentViewProps {
  client: ClientType;
  entries: CatalogEntry[];
  summary: StatusSummary | null;
  onToggle: (entry: CatalogEntry, client: ClientType, enable: boolean) => void;
  onEdit: (entry: CatalogEntry) => void;
  onDelete: (entry: CatalogEntry) => void;
}

export function ClientEnvironmentView({
  client,
  entries,
  summary,
  onToggle,
  onEdit,
  onDelete,
}: ClientEnvironmentViewProps) {
  const [scopeFilter, setScopeFilter] = useState<"all" | "global" | "project">("all");
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");

  const clientEntries = useMemo(
    () => filterEntriesByClient(entries, client),
    [entries, client],
  );

  const filteredCount = useMemo(
    () =>
      filterCatalogEntries(clientEntries, {
        query,
        scope: scopeFilter,
        status: statusFilter,
        focusClient: client,
      }).length,
    [clientEntries, query, scopeFilter, statusFilter, client],
  );

  return (
    <div className="space-y-4">
      <div>
        <p className="text-sm text-[var(--text-muted)]">Entorno</p>
        <h1 className="text-2xl font-semibold">{CLIENT_LABELS[client]}</h1>
      </div>

      <ClientStatusPanel
        client={client}
        summary={summary}
        catalogCount={clientEntries.length}
      />

      <CatalogToolbar
        query={query}
        onQueryChange={setQuery}
        statusFilter={statusFilter}
        onStatusFilterChange={setStatusFilter}
        scopeFilter={scopeFilter}
        onScopeFilterChange={setScopeFilter}
        totalCount={clientEntries.length}
        filteredCount={filteredCount}
        placeholder={`Buscar en ${CLIENT_LABELS[client]}…`}
      />

      <CatalogList
        entries={clientEntries}
        onToggle={onToggle}
        onEdit={onEdit}
        onDelete={onDelete}
        filterScope={scopeFilter}
        focusClient={client}
        query={query}
        statusFilter={statusFilter}
        emptyMessage={`No hay MCPs registrados para ${CLIENT_LABELS[client]}.`}
        noResultsMessage="Ningún MCP coincide. Prueba otro término o limpia los filtros."
      />
    </div>
  );
}
