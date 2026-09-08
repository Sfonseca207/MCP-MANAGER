# MCP Manager

Aplicación de escritorio para macOS que centraliza la administración de servidores MCP en **Claude Code**, **Cursor**, **Claude Desktop**, **VS Code** y **Windsurf**.

## Características

- Catálogo local permanente (SQLite) de todos los MCPs detectados o creados
- Activar/desactivar MCPs por cliente sin perder definiciones
- Scope global y por proyecto
- Backups automáticos antes de cada escritura + restauración
- File-watcher para detectar cambios externos en configs
- Reconciliación de drift (importar / ignorar / marcar deshabilitado)
- Contador estimado de tools con alerta del límite de Cursor (~40)
- Hard delete con doble confirmación

## Requisitos

- macOS 12+
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/) 1.70+ (`brew install rust`)

## Desarrollo

```bash
npm install
npm run tauri:dev
```

## Build `.app`

```bash
npm run tauri:build
```

La app se genera en:

```
src-tauri/target/release/bundle/macos/MCP Manager.app
```

## Instalación local

```bash
chmod +x scripts/install-local.sh
./scripts/install-local.sh
```

## Archivos que gestiona

| Cliente | Global | Proyecto |
|---------|--------|----------|
| Claude Code | `~/.claude.json` | `.mcp.json` + bloque `projects` |
| Cursor | `~/.cursor/mcp.json` | `.cursor/mcp.json` |
| Claude Desktop | `~/Library/Application Support/Claude/claude_desktop_config.json` | — |
| VS Code | Perfil usuario `mcp.json` | `.vscode/mcp.json` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` | — |

## Datos de la app

- Base de datos: `~/Library/Application Support/com.samuelfonseca.mcpmanager/catalog.db`
- Backups: `~/Library/Application Support/com.samuelfonseca.mcpmanager/backups/`

## Tests

```bash
cd src-tauri && cargo test
```

## Notas

- Tras habilitar/deshabilitar un MCP, **reinicia el cliente** correspondiente para que tome los cambios.
- La app modifica únicamente las claves `mcpServers` / `servers` en los archivos de config, preservando el resto del JSON.
- Los secretos (`env`/`headers`) no se loguean ni se envían a ningún servicio externo; se muestran en el formulario de edición tal como están en los archivos de config locales.
- Los backups guardan una copia del archivo de config completo (se conservan los últimos 20 por archivo).

## Licencia

[MIT](LICENSE)
