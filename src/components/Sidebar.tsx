import clsx from "clsx";
import {
  Archive,
  Code2,
  LayoutDashboard,
  Monitor,
  Plus,
  RefreshCw,
  Server,
  Settings,
  Sparkles,
  Terminal,
  Wind,
} from "lucide-react";
import type { ClientType } from "../lib/types";
import { ALL_CLIENTS, CLIENT_LABELS } from "../lib/types";
import appIcon from "../assets/app-icon-macos.png";

export type Page =
  | "dashboard"
  | "catalog"
  | "backups"
  | "settings"
  | `client:${ClientType}`;

interface SidebarProps {
  current: Page;
  onNavigate: (page: Page) => void;
  onAdd: () => void;
  onRefresh: () => void;
  activeCounts?: Record<string, number>;
}

const mainItems: { id: Page; label: string; icon: typeof LayoutDashboard }[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "catalog", label: "Catálogo", icon: Server },
  { id: "backups", label: "Backups", icon: Archive },
  { id: "settings", label: "Ajustes", icon: Settings },
];

const clientIcons: Record<ClientType, typeof Terminal> = {
  claude_code: Terminal,
  cursor: Code2,
  claude_desktop: Sparkles,
  vscode: Monitor,
  windsurf: Wind,
};

export function isClientPage(page: Page): page is `client:${ClientType}` {
  return page.startsWith("client:");
}

export function clientFromPage(page: Page): ClientType | null {
  if (!isClientPage(page)) return null;
  return page.replace("client:", "") as ClientType;
}

export function Sidebar({
  current,
  onNavigate,
  onAdd,
  onRefresh,
  activeCounts = {},
}: SidebarProps) {
  return (
    <aside className="flex h-full w-60 shrink-0 flex-col overflow-hidden border-r border-[var(--border)] bg-[var(--bg-elevated)]">
      {/* Navegación — scroll interno si hiciera falta */}
      <div className="min-h-0 flex-1 overflow-y-auto p-4 pb-2">
        <div className="mb-6">
          <div className="flex items-center gap-2 text-lg font-semibold">
            <img
              src={appIcon}
              alt=""
              className="h-6 w-6 rounded-md object-cover"
            />
            MCP Manager
          </div>
          <p className="mt-1 text-xs text-[var(--text-muted)]">
            Administra tus servidores MCP
          </p>
        </div>

        <nav className="flex flex-col gap-1">
          {mainItems.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => onNavigate(id)}
              className={clsx(
                "flex items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors",
                current === id
                  ? "bg-[var(--accent)] text-white"
                  : "text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]",
              )}
            >
              <Icon className="h-4 w-4 shrink-0" />
              {label}
            </button>
          ))}
        </nav>

        <div className="mt-6 mb-2 px-3 text-xs font-medium uppercase tracking-wide text-[var(--text-muted)]">
          Entornos
        </div>
        <nav className="flex flex-col gap-1">
          {ALL_CLIENTS.map((client) => {
            const pageId = `client:${client}` as Page;
            const Icon = clientIcons[client];
            const active = activeCounts[client] ?? 0;
            return (
              <button
                key={client}
                onClick={() => onNavigate(pageId)}
                className={clsx(
                  "flex items-center justify-between gap-2 rounded-lg px-3 py-2 text-sm transition-colors",
                  current === pageId
                    ? "bg-[var(--accent)] text-white"
                    : "text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]",
                )}
              >
                <span className="flex min-w-0 items-center gap-2">
                  <Icon className="h-4 w-4 shrink-0" />
                  <span className="truncate">{CLIENT_LABELS[client]}</span>
                </span>
                {active > 0 && (
                  <span
                    className={clsx(
                      "shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-medium",
                      current === pageId
                        ? "bg-white/20 text-white"
                        : "bg-[var(--success)]/15 text-[var(--success)]",
                    )}
                  >
                    {active}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Acciones — siempre visibles abajo */}
      <div className="shrink-0 border-t border-[var(--border)] bg-[var(--bg-elevated)] p-4">
        <div className="flex flex-col gap-2">
          <button
            onClick={onAdd}
            className="flex items-center justify-center gap-2 rounded-lg bg-[var(--accent)] px-3 py-2 text-sm font-medium text-white hover:opacity-90"
          >
            <Plus className="h-4 w-4" />
            Agregar MCP
          </button>
          <button
            onClick={onRefresh}
            className="flex items-center justify-center gap-2 rounded-lg border border-[var(--border)] px-3 py-2 text-sm text-[var(--text-muted)] hover:bg-[var(--bg-hover)]"
          >
            <RefreshCw className="h-4 w-4" />
            Refrescar
          </button>
        </div>
      </div>
    </aside>
  );
}
