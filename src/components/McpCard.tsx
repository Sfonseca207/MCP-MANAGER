import clsx from "clsx";
import { Pencil, Power, PowerOff, Trash2 } from "lucide-react";
import type { CatalogEntry, ClientType } from "../lib/types";
import { ALL_CLIENTS, CLIENT_LABELS } from "../lib/types";

function entryCommand(entry: CatalogEntry): string {
  if (entry.transport_type === "stdio") {
    return `${entry.command ?? ""} ${(entry.args ?? []).join(" ")}`.trim();
  }
  return entry.url ?? "";
}

interface McpCardProps {
  entry: CatalogEntry;
  focusClient?: ClientType;
  onToggle: (entry: CatalogEntry, client: ClientType, enable: boolean) => void;
  onEdit: (entry: CatalogEntry) => void;
  onDelete: (entry: CatalogEntry) => void;
}

export function McpCard({ entry, focusClient, onToggle, onEdit, onDelete }: McpCardProps) {
  const cmd = entryCommand(entry);
  const clientsToShow = focusClient ? [focusClient] : ALL_CLIENTS;

  return (
    <article
      className={clsx(
        "group flex flex-col rounded-xl border bg-[var(--bg-elevated)] transition-colors hover:border-[var(--accent)]/40",
        focusClient
          ? entry.client_states.find((s) => s.client === focusClient)?.enabled
            ? "border-[var(--success)]/25"
            : "border-[var(--border)]"
          : "border-[var(--border)]",
      )}
    >
      <div className="flex items-start justify-between gap-2 p-3 pb-2">
        <div className="min-w-0 flex-1">
          <h3 className="truncate font-semibold leading-tight" title={entry.display_name}>
            {entry.display_name}
          </h3>
          <div className="mt-1.5 flex flex-wrap gap-1">
            <span className="rounded-md bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--text-muted)]">
              {entry.transport_type}
            </span>
            <span className="rounded-md bg-[var(--bg-hover)] px-1.5 py-0.5 text-[10px] text-[var(--text-muted)]">
              {entry.scope_type === "global" ? "Global" : "Proyecto"}
            </span>
          </div>
        </div>
        <div className="flex shrink-0 gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            onClick={() => onEdit(entry)}
            className="rounded-md p-1 text-[var(--text-muted)] hover:bg-[var(--accent)]/10 hover:text-[var(--accent)]"
            title="Editar MCP"
          >
            <Pencil className="h-3.5 w-3.5" />
          </button>
          <button
            onClick={() => onDelete(entry)}
            className="rounded-md p-1 text-[var(--text-muted)] hover:bg-[var(--danger)]/10 hover:text-[var(--danger)]"
            title="Eliminar del catálogo"
          >
            <Trash2 className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      <p
        className="line-clamp-2 px-3 text-[11px] leading-relaxed text-[var(--text-muted)]"
        title={cmd}
      >
        {cmd}
      </p>

      {!focusClient && (
        <p className="mt-1 truncate px-3 text-[10px] text-[var(--text-muted)]">
          {CLIENT_LABELS[entry.client_origin]} ·{" "}
          {entry.source === "auto_detectado" ? "Auto" : "Manual"}
        </p>
      )}

      <div className="mt-auto p-3 pt-2">
        {focusClient ? (
          (() => {
            const state = entry.client_states.find((s) => s.client === focusClient);
            const enabled = state?.enabled ?? false;
            return (
              <button
                onClick={() => onToggle(entry, focusClient, !enabled)}
                className={clsx(
                  "flex w-full items-center justify-center gap-1.5 rounded-lg border py-2 text-xs font-medium transition-colors",
                  enabled
                    ? "border-[var(--success)]/30 bg-[var(--success)]/10 text-[var(--success)]"
                    : "border-[var(--border)] text-[var(--text-muted)] hover:bg-[var(--bg-hover)]",
                )}
              >
                {enabled ? <Power className="h-3.5 w-3.5" /> : <PowerOff className="h-3.5 w-3.5" />}
                {enabled ? "Activo" : "Inactivo"}
              </button>
            );
          })()
        ) : (
          <div className="flex flex-wrap gap-1">
            {clientsToShow.map((client) => {
              const state = entry.client_states.find((s) => s.client === client);
              const enabled = state?.enabled ?? false;
              return (
                <button
                  key={client}
                  onClick={() => onToggle(entry, client, !enabled)}
                  title={CLIENT_LABELS[client]}
                  className={clsx(
                    "rounded-md border px-1.5 py-1 text-[10px] transition-colors",
                    enabled
                      ? "border-[var(--success)]/30 bg-[var(--success)]/10 text-[var(--success)]"
                      : "border-[var(--border)] text-[var(--text-muted)] hover:bg-[var(--bg-hover)]",
                  )}
                >
                  {CLIENT_LABELS[client].split(" ")[0]}
                </button>
              );
            })}
          </div>
        )}
      </div>
    </article>
  );
}
