import type { BootstrapSummary } from "../lib/types";

interface OnboardingBannerProps {
  summary: BootstrapSummary | null;
}

export function OnboardingBanner({ summary }: OnboardingBannerProps) {
  if (!summary?.first_run) return null;

  return (
    <div className="mb-6 rounded-xl border border-[var(--accent)]/30 bg-[var(--accent)]/10 p-4">
      <h2 className="font-semibold">¡Bienvenido a MCP Manager!</h2>
      <p className="mt-1 text-sm text-[var(--text-muted)]">
        Se detectaron {summary.claude_code_count} MCPs en Claude Code,{" "}
        {summary.cursor_count} en Cursor, {summary.claude_desktop_count} en Claude Desktop,{" "}
        {summary.vscode_count} en VS Code y {summary.windsurf_count} en Windsurf.
      </p>
    </div>
  );
}
