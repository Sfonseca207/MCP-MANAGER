import clsx from "clsx";
import { AlertTriangle } from "lucide-react";
import type { ClientType, StatusSummary } from "../lib/types";

interface ClientStatusPanelProps {
  client: ClientType;
  summary: StatusSummary | null;
  catalogCount: number;
}

export function ClientStatusPanel({
  client,
  summary,
  catalogCount,
}: ClientStatusPanelProps) {
  if (!summary) return null;

  const active = summary.active_by_client[client] ?? 0;
  const tools = summary.estimated_tools_by_client[client] ?? 0;
  const isCursor = client === "cursor";
  const overLimit = isCursor && summary.cursor_over_limit;
  const limit = isCursor ? summary.cursor_tool_limit : null;
  const percent = limit ? Math.min((tools / limit) * 100, 100) : 0;

  return (
    <div className="flex flex-wrap items-stretch gap-2">
      <div className="flex min-w-[120px] flex-1 items-center gap-3 rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] px-4 py-3">
        <div>
          <p className="text-[11px] text-[var(--text-muted)]">Catálogo</p>
          <p className="text-xl font-semibold">{catalogCount}</p>
        </div>
      </div>
      <div className="flex min-w-[120px] flex-1 items-center gap-3 rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] px-4 py-3">
        <div>
          <p className="text-[11px] text-[var(--text-muted)]">Activos</p>
          <p className="text-xl font-semibold">{active}</p>
          <p className="text-[10px] text-[var(--text-muted)]">~{tools} tools</p>
        </div>
      </div>
      {isCursor && (
        <div className="flex min-w-[180px] flex-[2] flex-col justify-center rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] px-4 py-3">
          <div className="flex items-center justify-between">
            <p className="text-[11px] text-[var(--text-muted)]">Límite Cursor</p>
            {overLimit && <AlertTriangle className="h-3.5 w-3.5 text-[var(--warning)]" />}
          </div>
          <p className="text-sm font-semibold">
            {tools} / {limit}
          </p>
          <div className="mt-1.5 h-1.5 overflow-hidden rounded-full bg-[var(--bg-hover)]">
            <div
              className={clsx(
                "h-full rounded-full",
                overLimit ? "bg-[var(--danger)]" : "bg-[var(--accent)]",
              )}
              style={{ width: `${percent}%` }}
            />
          </div>
        </div>
      )}
    </div>
  );
}
