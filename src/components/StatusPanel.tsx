import clsx from "clsx";
import { AlertTriangle } from "lucide-react";
import type { StatusSummary } from "../lib/types";
import { CLIENT_LABELS } from "../lib/types";

interface StatusPanelProps {
  summary: StatusSummary | null;
}

export function StatusPanel({ summary }: StatusPanelProps) {
  if (!summary) return null;

  const cursorTools =
    summary.estimated_tools_by_client.cursor ?? 0;
  const cursorPercent = Math.min(
    (cursorTools / summary.cursor_tool_limit) * 100,
    100,
  );

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
        <p className="text-sm text-[var(--text-muted)]">Total en catálogo</p>
        <p className="mt-1 text-3xl font-semibold">{summary.total_catalog}</p>
      </div>

      {Object.entries(summary.active_by_client).map(([client, count]) => (
        <div
          key={client}
          className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4"
        >
          <p className="text-sm text-[var(--text-muted)]">
            Activos — {CLIENT_LABELS[client as keyof typeof CLIENT_LABELS] ?? client}
          </p>
          <p className="mt-1 text-3xl font-semibold">{count}</p>
          <p className="mt-1 text-xs text-[var(--text-muted)]">
            ~{summary.estimated_tools_by_client[client] ?? 0} tools estimadas
          </p>
        </div>
      ))}

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4 md:col-span-2 xl:col-span-1">
        <div className="flex items-center justify-between">
          <p className="text-sm text-[var(--text-muted)]">Límite Cursor (~40 tools)</p>
          {summary.cursor_over_limit && (
            <AlertTriangle className="h-4 w-4 text-[var(--warning)]" />
          )}
        </div>
        <p className="mt-1 text-lg font-semibold">
          {cursorTools} / {summary.cursor_tool_limit}
        </p>
        <div className="mt-3 h-2 overflow-hidden rounded-full bg-[var(--bg-hover)]">
          <div
            className={clsx(
              "h-full rounded-full transition-all",
              summary.cursor_over_limit ? "bg-[var(--danger)]" : "bg-[var(--accent)]",
            )}
            style={{ width: `${cursorPercent}%` }}
          />
        </div>
        {summary.cursor_over_limit && (
          <p className="mt-2 text-xs text-[var(--warning)]">
            Has superado el límite práctico de Cursor. Algunas tools pueden no estar disponibles.
          </p>
        )}
      </div>
    </div>
  );
}
