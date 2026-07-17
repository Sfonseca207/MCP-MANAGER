interface ToastProps {
  message: string | null;
  onClose: () => void;
}

export function Toast({ message, onClose }: ToastProps) {
  if (!message) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 max-w-md rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4 shadow-lg">
      <p className="text-sm">{message}</p>
      <button
        onClick={onClose}
        className="mt-2 text-xs text-[var(--accent)] hover:underline"
      >
        Cerrar
      </button>
    </div>
  );
}
