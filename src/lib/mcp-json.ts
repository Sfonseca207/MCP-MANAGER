import type { CatalogEntry, TransportType } from "./types";

export interface McpDefinitionFields {
  transport_type: TransportType;
  command?: string | null;
  args?: string[] | null;
  env?: Record<string, string> | null;
  url?: string | null;
  headers?: Record<string, string> | null;
}

export interface ParsedMcpJson extends McpDefinitionFields {
  displayName?: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseStringMap(value: unknown): Record<string, string> | null {
  if (!isRecord(value)) return null;
  const result: Record<string, string> = {};
  for (const [k, v] of Object.entries(value)) {
    if (typeof v === "string") result[k] = v;
  }
  return Object.keys(result).length ? result : null;
}

function inferTransport(def: Record<string, unknown>): TransportType {
  const explicit = def.type;
  if (explicit === "http") return "http";
  if (explicit === "sse") return "sse";
  if (explicit === "stdio") return "stdio";
  if ("url" in def || "serverUrl" in def) {
    return explicit === "sse" ? "sse" : "http";
  }
  return "stdio";
}

function parseDefinitionObject(def: Record<string, unknown>): McpDefinitionFields {
  const transport_type = inferTransport(def);
  const command =
    typeof def.command === "string" ? def.command : null;
  const args = Array.isArray(def.args)
    ? def.args.filter((a): a is string => typeof a === "string")
    : null;
  const env = parseStringMap(def.env);
  const url =
    typeof def.url === "string"
      ? def.url
      : typeof def.serverUrl === "string"
        ? def.serverUrl
        : null;
  const headers = parseStringMap(def.headers);

  if (transport_type === "stdio" && !command) {
    throw new Error('Definición stdio: falta "command"');
  }
  if (transport_type !== "stdio" && !url) {
    throw new Error('Definición remota: falta "url" o "serverUrl"');
  }

  return {
    transport_type,
    command: transport_type === "stdio" ? command : null,
    args: transport_type === "stdio" ? (args?.length ? args : null) : null,
    env,
    url: transport_type !== "stdio" ? url : null,
    headers: transport_type !== "stdio" ? headers : null,
  };
}

function looksLikeDefinition(obj: Record<string, unknown>): boolean {
  return (
    "command" in obj ||
    "url" in obj ||
    "serverUrl" in obj ||
    "type" in obj ||
    "args" in obj
  );
}

function unwrapMcpJson(
  parsed: Record<string, unknown>,
  fallbackName?: string,
): { displayName?: string; definition: Record<string, unknown> } {
  if (isRecord(parsed.mcpServers)) {
    const keys = Object.keys(parsed.mcpServers);
    if (!keys.length) throw new Error("mcpServers está vacío");
    if (keys.length > 1) {
      throw new Error(
        `Solo un MCP por JSON. Encontrados: ${keys.join(", ")}`,
      );
    }
    const inner = parsed.mcpServers[keys[0]];
    if (!isRecord(inner)) throw new Error("Entrada de mcpServers inválida");
    return { displayName: keys[0], definition: inner };
  }

  const keys = Object.keys(parsed);
  if (keys.length === 1) {
    const inner = parsed[keys[0]];
    if (isRecord(inner) && looksLikeDefinition(inner)) {
      return { displayName: keys[0], definition: inner };
    }
  }

  if (!looksLikeDefinition(parsed)) {
    throw new Error(
      "JSON no reconocido. Pega la definición del MCP o un bloque mcpServers.",
    );
  }

  return { displayName: fallbackName, definition: parsed };
}

export function parseMcpJsonText(
  text: string,
  fallbackName?: string,
): ParsedMcpJson {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("JSON inválido: revisa comillas, comas y llaves");
  }

  if (!isRecord(parsed)) {
    throw new Error("El JSON debe ser un objeto");
  }

  const { displayName, definition } = unwrapMcpJson(parsed, fallbackName);
  const fields = parseDefinitionObject(definition);

  return {
    displayName,
    ...fields,
  };
}

export function buildMcpDefinitionJson(
  fields: McpDefinitionFields,
): Record<string, unknown> {
  const obj: Record<string, unknown> = { type: fields.transport_type };

  if (fields.transport_type === "stdio") {
    if (fields.command) obj.command = fields.command;
    if (fields.args?.length) obj.args = fields.args;
    if (fields.env && Object.keys(fields.env).length) obj.env = fields.env;
  } else {
    if (fields.url) obj.url = fields.url;
    if (fields.headers && Object.keys(fields.headers).length) {
      obj.headers = fields.headers;
    }
  }

  return obj;
}

export function entryToMcpJsonText(entry: CatalogEntry): string {
  return JSON.stringify(
    buildMcpDefinitionJson({
      transport_type: entry.transport_type,
      command: entry.command,
      args: entry.args,
      env: entry.env,
      url: entry.url,
      headers: entry.headers,
    }),
    null,
    2,
  );
}

export function formToMcpJsonText(fields: McpDefinitionFields): string {
  return JSON.stringify(buildMcpDefinitionJson(fields), null, 2);
}

export const MCP_JSON_PLACEHOLDER = `{
  "type": "stdio",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-filesystem"]
}`;
