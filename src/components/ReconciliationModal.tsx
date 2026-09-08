import type { ReconciliationConflict } from "../lib/types";
import { CLIENT_LABELS } from "../lib/types";

interface ReconciliationModalProps {
  conflicts: ReconciliationConflict[];
  onResolve: (conflict: ReconciliationConflict, action: "import" | "mark_disabled" | "ignore") => void;
  onClose: () => void;
}

const kindLabels: Record<string, string> = {
  new_external: "Nuevo MCP externo",
  removed_external: "Eliminado externamente",
  definition_changed: "Definición cambiada",
  name_collision: "Colisión de nombre",
};

export function ReconciliationModal({
  conflicts,
  onResolve,
  onClose,
}: ReconciliationModalProps) {
  if (conflicts.length === 0) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div className="max-h-[80vh] w-full max-w-2xl overflow-y-auto rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-6">
        <h2 className="text-lg font-semibold">Cambios externos detectados</h2>
        <p className="mt-1 text-sm text-[var(--text-muted)]">
          Los archivos de configuración fueron modificados fuera de MCP Manager.
        </p>

        <div className="mt-4 space-y-3">
          {conflicts.map((conflict) => (
            <div
              key={conflict.id}
              className="rounded-lg border border-[var(--border)] p-4"
            >
              <p className="font-medium">{conflict.server_name}</p>
              <p className="mt-1 text-xs text-[var(--text-muted)]">
                {kindLabels[conflict.kind] ?? conflict.kind} ·{" "}
                {CLIENT_LABELS[conflict.client]} · {conflict.scope_type}
              </p>
              <div className="mt-3 flex gap-2">
                {conflict.kind === "new_external" && (
                  <button
                    onClick={() => onResolve(conflict, "import")}
                    className="rounded-lg bg-[var(--accent)] px-3 py-1 text-xs text-[var(--on-accent)]"
                  >
                    Importar
                  </button>
                )}
                {conflict.kind === "removed_external" && (
                  <button
                    onClick={() => onResolve(conflict, "mark_disabled")}
                    className="rounded-lg bg-[var(--warning)]/20 px-3 py-1 text-xs text-[var(--warning)]"
                  >
                    Marcar deshabilitado
                  </button>
                )}
                {conflict.kind === "definition_changed" && (
                  <button
                    onClick={() => onResolve(conflict, "import")}
                    className="rounded-lg bg-[var(--accent)] px-3 py-1 text-xs text-[var(--on-accent)]"
                  >
                    Actualizar catálogo
                  </button>
                )}
                <button
                  onClick={() => onResolve(conflict, "ignore")}
                  className="rounded-lg border border-[var(--border)] px-3 py-1 text-xs"
                >
                  Ignorar
                </button>
              </div>
            </div>
          ))}
        </div>

        <div className="mt-6 flex justify-end">
          <button
            onClick={onClose}
            className="rounded-lg border border-[var(--border)] px-4 py-2 text-sm"
          >
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
}
