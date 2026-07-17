import { useEffect, useState } from "react";
import type {
  CatalogEntry,
  ClientType,
  CreateMcpInput,
  ScopeType,
  TransportType,
  UpdateMcpInput,
} from "../lib/types";
import { ALL_CLIENTS, CLIENT_LABELS } from "../lib/types";
import {
  entryToMcpJsonText,
  formToMcpJsonText,
  MCP_JSON_PLACEHOLDER,
  parseMcpJsonText,
} from "../lib/mcp-json";
import { SegmentedControl, Select } from "./Select";

type McpFormMode = "create" | "edit";
type InputMode = "fields" | "json";

interface McpFormProps {
  open: boolean;
  mode: McpFormMode;
  entry?: CatalogEntry | null;
  onClose: () => void;
  onCreate?: (input: CreateMcpInput) => Promise<void>;
  onUpdate?: (input: UpdateMcpInput) => Promise<void>;
  defaultClient?: ClientType | null;
}

function recordToLines(record?: Record<string, string> | null): string {
  if (!record) return "";
  return Object.entries(record)
    .map(([k, v]) => `${k}=${v}`)
    .join("\n");
}

function parseKeyValueLines(text: string): Record<string, string> | null {
  if (!text.trim()) return null;
  const result: Record<string, string> = {};
  for (const line of text.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    const [key, ...rest] = trimmed.split("=");
    if (key && rest.length) result[key.trim()] = rest.join("=").trim();
  }
  return Object.keys(result).length ? result : null;
}

function fieldsFromForm(
  transportType: TransportType,
  command: string,
  args: string,
  url: string,
  envText: string,
  headersText: string,
) {
  return {
    transport_type: transportType,
    command: transportType === "stdio" ? command : null,
    args:
      transportType === "stdio" ? args.split(" ").filter(Boolean) : null,
    env: parseKeyValueLines(envText),
    url: transportType !== "stdio" ? url : null,
    headers:
      transportType !== "stdio" ? parseKeyValueLines(headersText) : null,
  };
}

export function McpForm({
  open,
  mode,
  entry,
  onClose,
  onCreate,
  onUpdate,
  defaultClient,
}: McpFormProps) {
  const [inputMode, setInputMode] = useState<InputMode>("fields");
  const [displayName, setDisplayName] = useState("");
  const [transportType, setTransportType] = useState<TransportType>("stdio");
  const [command, setCommand] = useState("npx");
  const [args, setArgs] = useState("");
  const [url, setUrl] = useState("");
  const [envText, setEnvText] = useState("");
  const [headersText, setHeadersText] = useState("");
  const [jsonText, setJsonText] = useState(MCP_JSON_PLACEHOLDER);
  const [jsonError, setJsonError] = useState<string | null>(null);
  const [clientOrigin, setClientOrigin] = useState<ClientType>(
    defaultClient ?? "cursor",
  );
  const [scopeType, setScopeType] = useState<ScopeType>("global");
  const [scopePath, setScopePath] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open) return;

    setInputMode("fields");
    setJsonError(null);

    if (mode === "edit" && entry) {
      setDisplayName(entry.display_name);
      setTransportType(entry.transport_type);
      setCommand(entry.command ?? "npx");
      setArgs((entry.args ?? []).join(" "));
      setUrl(entry.url ?? "");
      setEnvText(recordToLines(entry.env));
      setHeadersText(recordToLines(entry.headers));
      setJsonText(entryToMcpJsonText(entry));
      setClientOrigin(entry.client_origin);
      setScopeType(entry.scope_type);
      setScopePath(entry.scope_path ?? "");
    } else {
      setDisplayName("");
      setTransportType("stdio");
      setCommand("npx");
      setArgs("");
      setUrl("");
      setEnvText("");
      setHeadersText("");
      setJsonText(MCP_JSON_PLACEHOLDER);
      setClientOrigin(defaultClient ?? "cursor");
      setScopeType("global");
      setScopePath("");
    }
  }, [open, mode, entry, defaultClient]);

  if (!open) return null;

  const isEdit = mode === "edit";
  const isActive = entry?.client_states.some((s) => s.enabled) ?? false;

  const applyParsedToForm = (parsed: ReturnType<typeof parseMcpJsonText>) => {
    if (parsed.displayName) setDisplayName(parsed.displayName);
    setTransportType(parsed.transport_type);
    setCommand(parsed.command ?? "npx");
    setArgs((parsed.args ?? []).join(" "));
    setUrl(parsed.url ?? "");
    setEnvText(recordToLines(parsed.env));
    setHeadersText(recordToLines(parsed.headers));
  };

  const switchToJson = () => {
    setJsonText(
      formToMcpJsonText(
        fieldsFromForm(
          transportType,
          command,
          args,
          url,
          envText,
          headersText,
        ),
      ),
    );
    setJsonError(null);
    setInputMode("json");
  };

  const switchToFields = () => {
    try {
      const parsed = parseMcpJsonText(jsonText, displayName || undefined);
      applyParsedToForm(parsed);
      setJsonError(null);
      setInputMode("fields");
    } catch (err) {
      setJsonError(
        err instanceof Error ? err.message : "No se pudo interpretar el JSON",
      );
    }
  };

  const resolvePayload = (): {
    display_name: string;
    transport_type: TransportType;
    command?: string | null;
    args?: string[] | null;
    env?: Record<string, string> | null;
    url?: string | null;
    headers?: Record<string, string> | null;
  } => {
    if (inputMode === "json") {
      const parsed = parseMcpJsonText(jsonText, displayName || undefined);
      const name = parsed.displayName ?? displayName;
      if (!name.trim()) {
        throw new Error("Indica un nombre para el MCP");
      }
      return {
        display_name: name.trim(),
        transport_type: parsed.transport_type,
        command: parsed.command,
        args: parsed.args,
        env: parsed.env,
        url: parsed.url,
        headers: parsed.headers,
      };
    }

    if (!displayName.trim()) {
      throw new Error("Indica un nombre para el MCP");
    }

    return {
      display_name: displayName.trim(),
      ...fieldsFromForm(
        transportType,
        command,
        args,
        url,
        envText,
        headersText,
      ),
    };
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setJsonError(null);
    try {
      const payload = resolvePayload();

      if (isEdit && entry && onUpdate) {
        await onUpdate({
          id: entry.id,
          display_name: payload.display_name,
          transport_type: payload.transport_type,
          command: payload.command,
          args: payload.args,
          env: payload.env,
          url: payload.url,
          headers: payload.headers,
        });
      } else if (onCreate) {
        await onCreate({
          display_name: payload.display_name,
          transport_type: payload.transport_type,
          command: payload.command,
          args: payload.args,
          env: payload.env,
          url: payload.url,
          headers: payload.headers,
          client_origin: clientOrigin,
          scope_type: scopeType,
          scope_path: scopeType === "project" ? scopePath : null,
        });
      }
      onClose();
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Error al guardar el MCP";
      setJsonError(message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <form
        onSubmit={handleSubmit}
        className="max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-6"
      >
        <h2 className="text-lg font-semibold">
          {isEdit ? "Editar MCP" : "Agregar MCP"}
        </h2>
        <p className="mt-1 text-sm text-[var(--text-muted)]">
          {isEdit
            ? isActive
              ? "Los cambios se aplicarán también en los archivos de config donde esté activo."
              : "Actualiza la definición guardada en el catálogo."
            : "Se guardará deshabilitado hasta que lo actives en un cliente."}
        </p>

        <div className="mt-4 flex rounded-lg border border-[var(--border)] p-0.5">
          <button
            type="button"
            onClick={() => (inputMode === "json" ? switchToFields() : undefined)}
            className={`flex-1 rounded-md px-3 py-1.5 text-sm transition-colors ${
              inputMode === "fields"
                ? "bg-[var(--accent)] text-white"
                : "text-[var(--text-muted)] hover:text-[var(--text)]"
            }`}
          >
            Campos
          </button>
          <button
            type="button"
            onClick={() => (inputMode === "fields" ? switchToJson() : undefined)}
            className={`flex-1 rounded-md px-3 py-1.5 text-sm transition-colors ${
              inputMode === "json"
                ? "bg-[var(--accent)] text-white"
                : "text-[var(--text-muted)] hover:text-[var(--text)]"
            }`}
          >
            JSON
          </button>
        </div>

        <div className="mt-4 space-y-3">
          <label className="block text-sm">
            Nombre
            <input
              required={inputMode === "fields"}
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder={
                inputMode === "json"
                  ? "Opcional si el JSON incluye la clave del servidor"
                  : undefined
              }
              className="mt-1 w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
            />
          </label>

          {inputMode === "fields" ? (
            <>
              <div className="block text-sm">
                <span className="mb-1 block">Transporte</span>
                <SegmentedControl
                  value={transportType}
                  onChange={setTransportType}
                  options={[
                    { value: "stdio", label: "stdio" },
                    { value: "http", label: "http" },
                    { value: "sse", label: "sse" },
                  ]}
                />
              </div>

              {transportType === "stdio" ? (
                <>
                  <label className="block text-sm">
                    Command
                    <input
                      value={command}
                      onChange={(e) => setCommand(e.target.value)}
                      className="mt-1 w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
                    />
                  </label>
                  <label className="block text-sm">
                    Args (separados por espacio)
                    <input
                      value={args}
                      onChange={(e) => setArgs(e.target.value)}
                      className="mt-1 w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2 font-mono text-xs"
                    />
                  </label>
                </>
              ) : (
                <>
                  <label className="block text-sm">
                    URL
                    <input
                      required
                      value={url}
                      onChange={(e) => setUrl(e.target.value)}
                      className="mt-1 w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
                    />
                  </label>
                  <label className="block text-sm">
                    Headers (KEY=VALUE, una por línea)
                    <textarea
                      value={headersText}
                      onChange={(e) => setHeadersText(e.target.value)}
                      rows={2}
                      className="mt-1 w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2 font-mono text-xs"
                      placeholder="Authorization=Bearer TOKEN"
                    />
                  </label>
                </>
              )}

              <label className="block text-sm">
                Variables de entorno (KEY=VALUE, una por línea)
                <textarea
                  value={envText}
                  onChange={(e) => setEnvText(e.target.value)}
                  rows={3}
                  className="mt-1 w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2 font-mono text-xs"
                  placeholder="API_KEY=valor"
                />
              </label>
            </>
          ) : (
            <div>
              <label className="block text-sm">
                Definición JSON del MCP
                <textarea
                  value={jsonText}
                  onChange={(e) => {
                    setJsonText(e.target.value);
                    setJsonError(null);
                  }}
                  rows={14}
                  spellCheck={false}
                  className="mt-1 w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2 font-mono text-xs leading-relaxed"
                />
              </label>
              <p className="mt-2 text-xs text-[var(--text-muted)]">
                Pega la definición tal como va en{" "}
                <code className="rounded bg-[var(--bg)] px-1">mcpServers</code>,
                un bloque{" "}
                <code className="rounded bg-[var(--bg)] px-1">
                  {"{ \"nombre\": { ... } }"}
                </code>{" "}
                o un objeto{" "}
                <code className="rounded bg-[var(--bg)] px-1">mcpServers</code>{" "}
                completo.
              </p>
            </div>
          )}

          {!isEdit && (
            <>
              <div className="grid grid-cols-2 gap-3">
                <label className="block text-sm">
                  Cliente origen
                  <Select
                    className="mt-1"
                    value={clientOrigin}
                    onChange={setClientOrigin}
                    options={ALL_CLIENTS.map((c) => ({
                      value: c,
                      label: CLIENT_LABELS[c],
                    }))}
                  />
                </label>
                <div className="block text-sm">
                  <span className="mb-1 block">Scope</span>
                  <SegmentedControl
                    value={scopeType}
                    onChange={setScopeType}
                    options={[
                      { value: "global", label: "Global" },
                      { value: "project", label: "Proyecto" },
                    ]}
                  />
                </div>
              </div>

              {scopeType === "project" && (
                <label className="block text-sm">
                  Ruta del proyecto
                  <input
                    required
                    value={scopePath}
                    onChange={(e) => setScopePath(e.target.value)}
                    placeholder="/Users/tu/proyecto"
                    className="mt-1 w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
                  />
                </label>
              )}
            </>
          )}

          {isEdit && entry && (
            <div className="rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[var(--text-muted)]">
              {CLIENT_LABELS[entry.client_origin]} ·{" "}
              {entry.scope_type === "global"
                ? "Global"
                : `Proyecto: ${entry.scope_path}`}
            </div>
          )}

          {jsonError && (
            <p className="rounded-lg border border-[var(--danger)]/30 bg-[var(--danger)]/10 px-3 py-2 text-sm text-[var(--danger)]">
              {jsonError}
            </p>
          )}
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg border border-[var(--border)] px-4 py-2 text-sm"
          >
            Cancelar
          </button>
          <button
            type="submit"
            disabled={loading}
            className="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
          >
            {loading ? "Guardando..." : isEdit ? "Guardar cambios" : "Guardar"}
          </button>
        </div>
      </form>
    </div>
  );
}
