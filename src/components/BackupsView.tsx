import type { BackupRecord } from "../lib/types";
import { CLIENT_LABELS } from "../lib/types";

interface BackupsViewProps {
  backups: BackupRecord[];
  onRestore: (id: string) => void;
}

export function BackupsView({ backups, onRestore }: BackupsViewProps) {
  if (backups.length === 0) {
    return (
      <div className="rounded-xl border border-dashed border-[var(--border)] p-8 text-center text-[var(--text-muted)]">
        No hay backups todavía. Se crean automáticamente antes de cada cambio.
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      {backups.map((backup) => (
        <div
          key={backup.id}
          className="flex items-center justify-between rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4"
        >
          <div>
            <p className="font-medium">
              {CLIENT_LABELS[backup.client]} — {backup.reason}
            </p>
            <p className="mt-1 text-xs text-[var(--text-muted)]">
              {new Date(backup.created_at).toLocaleString()} · {backup.file_path}
            </p>
          </div>
          <button
            onClick={() => onRestore(backup.id)}
            className="rounded-lg bg-[var(--accent)] px-3 py-1.5 text-sm text-white hover:opacity-90"
          >
            Restaurar
          </button>
        </div>
      ))}
    </div>
  );
}
