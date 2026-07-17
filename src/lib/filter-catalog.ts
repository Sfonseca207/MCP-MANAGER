import type { CatalogEntry, ClientType } from "./types";

export type StatusFilter = "all" | "active" | "inactive";

export function entrySearchText(entry: CatalogEntry): string {
  return [
    entry.display_name,
    entry.command,
    ...(entry.args ?? []),
    entry.url,
    entry.scope_path,
    entry.transport_type,
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
}

export function isEntryActive(
  entry: CatalogEntry,
  focusClient?: ClientType,
): boolean {
  if (focusClient) {
    return (
      entry.client_states.find((s) => s.client === focusClient)?.enabled ?? false
    );
  }
  return entry.client_states.some((s) => s.enabled);
}

export function filterCatalogEntries(
  entries: CatalogEntry[],
  options: {
    query?: string;
    scope?: "all" | "global" | "project";
    status?: StatusFilter;
    focusClient?: ClientType;
  },
): CatalogEntry[] {
  const query = options.query?.trim().toLowerCase() ?? "";

  return entries.filter((entry) => {
    if (options.scope && options.scope !== "all" && entry.scope_type !== options.scope) {
      return false;
    }

    if (query && !entrySearchText(entry).includes(query)) {
      return false;
    }

    if (options.status && options.status !== "all") {
      const active = isEntryActive(entry, options.focusClient);
      if (options.status === "active" && !active) return false;
      if (options.status === "inactive" && active) return false;
    }

    return true;
  });
}
