import clsx from "clsx";
import { Search, X } from "lucide-react";
import type { StatusFilter } from "../lib/filter-catalog";
import { SegmentedControl } from "./Select";

interface CatalogToolbarProps {
  query: string;
  onQueryChange: (query: string) => void;
  statusFilter: StatusFilter;
  onStatusFilterChange: (status: StatusFilter) => void;
  scopeFilter?: "all" | "global" | "project";
  onScopeFilterChange?: (scope: "all" | "global" | "project") => void;
  totalCount: number;
  filteredCount: number;
  placeholder?: string;
}

const scopeOptions = [
  { value: "all" as const, label: "Todos" },
  { value: "global" as const, label: "Global" },
  { value: "project" as const, label: "Proyecto" },
];

const statusOptions: { value: StatusFilter; label: string }[] = [
  { value: "all", label: "Todos" },
  { value: "active", label: "Activos" },
  { value: "inactive", label: "Inactivos" },
];

export function CatalogToolbar({
  query,
  onQueryChange,
  statusFilter,
  onStatusFilterChange,
  scopeFilter,
  onScopeFilterChange,
  totalCount,
  filteredCount,
  placeholder = "Buscar por nombre, comando, URL…",
}: CatalogToolbarProps) {
  return (
    <div className="sticky top-0 z-10 -mx-6 border-b border-[var(--border)] bg-[var(--bg)]/95 px-6 py-3 backdrop-blur-sm">
      <div className="flex flex-wrap items-center gap-2">
        <div className="relative min-w-[200px] flex-1">
          <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--text-muted)]" />
          <input
            type="search"
            value={query}
            onChange={(e) => onQueryChange(e.target.value)}
            placeholder={placeholder}
            className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] py-2 pl-9 pr-9 text-sm outline-none focus:border-[var(--accent)]"
          />
          {query && (
            <button
              onClick={() => onQueryChange("")}
              className="absolute right-2 top-1/2 -translate-y-1/2 rounded p-1 text-[var(--text-muted)] hover:bg-[var(--bg-hover)]"
              aria-label="Limpiar búsqueda"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
        </div>

        <div className="flex rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] p-0.5">
          {statusOptions.map(({ value, label }) => (
            <button
              key={value}
              onClick={() => onStatusFilterChange(value)}
              className={clsx(
                "rounded-md px-3 py-1.5 text-xs transition-colors",
                statusFilter === value
                  ? "bg-[var(--accent)] text-[var(--on-accent)]"
                  : "text-[var(--text-muted)] hover:text-[var(--text)]",
              )}
            >
              {label}
            </button>
          ))}
        </div>

        {scopeFilter !== undefined && onScopeFilterChange && (
          <SegmentedControl
            value={scopeFilter}
            onChange={onScopeFilterChange}
            options={scopeOptions}
            size="sm"
          />
        )}
      </div>

      <p className="mt-2 text-xs text-[var(--text-muted)]">
        {filteredCount === totalCount
          ? `${totalCount} MCP${totalCount !== 1 ? "s" : ""}`
          : `Mostrando ${filteredCount} de ${totalCount}`}
      </p>
    </div>
  );
}
