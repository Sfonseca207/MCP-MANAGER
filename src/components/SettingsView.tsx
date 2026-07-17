import { open } from "@tauri-apps/plugin-dialog";
import { FolderPlus, Trash2 } from "lucide-react";
import type { WatchedProject } from "../lib/types";

interface SettingsViewProps {
  projects: WatchedProject[];
  onAddProject: (path: string) => Promise<void>;
  onRemoveProject: (id: string) => Promise<void>;
}

export function SettingsView({
  projects,
  onAddProject,
  onRemoveProject,
}: SettingsViewProps) {
  const pickFolder = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      await onAddProject(selected);
    }
  };

  return (
    <div className="space-y-6">
      <section>
        <h2 className="text-lg font-semibold">Proyectos vigilados</h2>
        <p className="mt-1 text-sm text-[var(--text-muted)]">
          MCPs de scope proyecto en Claude Code, Cursor y VS Code.
        </p>

        <button
          onClick={pickFolder}
          className="mt-4 flex items-center gap-2 rounded-lg bg-[var(--accent)] px-4 py-2 text-sm text-white"
        >
          <FolderPlus className="h-4 w-4" />
          Agregar proyecto
        </button>

        <div className="mt-4 space-y-2">
          {projects.length === 0 ? (
            <p className="text-sm text-[var(--text-muted)]">
              No hay proyectos vigilados.
            </p>
          ) : (
            projects.map((project) => (
              <div
                key={project.id}
                className="flex items-center justify-between rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] px-4 py-3"
              >
                <span className="truncate text-sm">{project.path}</span>
                <button
                  onClick={() => onRemoveProject(project.id)}
                  className="rounded p-1 text-[var(--text-muted)] hover:text-[var(--danger)]"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            ))
          )}
        </div>
      </section>

      <section className="rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
        <h3 className="font-medium">Clientes soportados</h3>
        <ul className="mt-2 space-y-1 text-sm text-[var(--text-muted)]">
          <li>Claude Code — ~/.claude.json (global) + .mcp.json (proyecto)</li>
          <li>Cursor — ~/.cursor/mcp.json (global) + .cursor/mcp.json (proyecto)</li>
          <li>Claude Desktop — ~/Library/Application Support/Claude/</li>
          <li>VS Code — perfil usuario + .vscode/mcp.json</li>
          <li>Windsurf — ~/.codeium/windsurf/mcp_config.json</li>
        </ul>
      </section>
    </div>
  );
}
