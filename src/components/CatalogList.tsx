import {
  filterCatalogEntries,
  type StatusFilter,
} from "../lib/filter-catalog";
import type { CatalogEntry, ClientType } from "../lib/types";
import { McpCard } from "./McpCard";

interface CatalogListProps {
  entries: CatalogEntry[];
  onToggle: (entry: CatalogEntry, client: ClientType, enable: boolean) => void;
  onEdit: (entry: CatalogEntry) => void;
  onDelete: (entry: CatalogEntry) => void;
  filterScope?: "all" | "global" | "project";
  focusClient?: ClientType;
  query?: string;
  statusFilter?: StatusFilter;
  emptyMessage?: string;
  noResultsMessage?: string;
}

export function CatalogList({
  entries,
  onToggle,
  onEdit,
  onDelete,
  filterScope = "all",
  focusClient,
  query = "",
  statusFilter = "all",
  emptyMessage = "No hay MCPs en el catálogo.",
  noResultsMessage = "Ningún MCP coincide con tu búsqueda.",
}: CatalogListProps) {
  const filtered = filterCatalogEntries(entries, {
    query,
    scope: filterScope,
    status: statusFilter,
    focusClient,
  });

  if (entries.length === 0) {
    return (
      <div className="col-span-full rounded-xl border border-dashed border-[var(--border)] p-8 text-center text-[var(--text-muted)]">
        {emptyMessage}
      </div>
    );
  }

  if (filtered.length === 0) {
    return (
      <div className="col-span-full rounded-xl border border-dashed border-[var(--border)] p-8 text-center text-[var(--text-muted)]">
        {noResultsMessage}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {filtered.map((entry) => (
        <McpCard
          key={entry.id}
          entry={entry}
          focusClient={focusClient}
          onToggle={onToggle}
          onEdit={onEdit}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
