# Requerimientos — MCP Manager (macOS)

## 1. Contexto y objetivo

Aplicación personal de escritorio para macOS que centraliza la administración de servidores MCP (Model Context Protocol) instalados en **Claude Code** y **Cursor**. El objetivo es tener un catálogo permanente de todos los MCP que alguna vez se configuraron, y poder **activar/desactivar** cada uno por cliente sin perder su definición — evitando así que MCPs no usados consuman contexto y recursos al arrancar Claude Code o Cursor.

Principio central: **el catálogo local es la fuente de verdad de "qué existe"; los archivos JSON reales de cada cliente son la fuente de verdad de "qué está corriendo ahora".**

## 2. Alcance de la V1

- Plataforma: macOS, app de escritorio local, sin backend en la nube.
- Clientes soportados: Claude Code y Cursor.
- Persistencia: local (SQLite embebido recomendado).
- **Supuesto de alcance** (a confirmar, ver sección 12): V1 opera solo sobre la configuración **global/de usuario** de cada cliente, no sobre configuraciones por proyecto.

## 3. Glosario

| Término | Significado |
|---|---|
| MCP | Servidor Model Context Protocol; expone herramientas/datos a Claude Code o Cursor |
| `mcpServers` | Clave JSON estándar donde cada cliente lista sus servidores MCP, con `command/args/env` (stdio) o `url/headers` (http/sse) |
| Catálogo local | Base de datos propia de la app: todo MCP alguna vez visto o creado, esté o no activo |
| Habilitar | Insertar la definición del MCP en el JSON real del cliente |
| Deshabilitar (soft delete) | Quitar la entrada del JSON real, sin borrar el registro del catálogo |
| Eliminar (hard delete) | Borrar el registro del catálogo de forma permanente |

## 4. Fuentes reales de configuración

Esto es lo que la app va a leer y escribir directamente. Los nombres de archivo son distintos a los que mencionaste ("Cloud Code", "Jason") — dejo aquí los correctos para que sirvan de referencia técnica del proyecto:

**Claude Code**
- Configuración global/usuario: `~/.claude.json`, dentro de la clave `mcpServers` en el nivel raíz del archivo. Esto es lo que se carga siempre, sin importar el proyecto abierto — es el bloque que te interesa para "gasto de recursos al abrir Claude Code".
- Ese mismo archivo también guarda, dentro de `projects."<ruta-absoluta-del-proyecto>".mcpServers`, servidores locales atados a un proyecto específico (fuera del alcance V1, ver sección 12).
- Existe además `.mcp.json` en la raíz de cada repo, para servidores de proyecto compartidos vía git (también fuera de alcance V1).
- ⚠️ Importante: `~/.claude.json` no contiene solo MCPs — también guarda historial, estado de onboarding, cuenta OAuth, etc. La app debe tocar **únicamente** la clave `mcpServers`, sin alterar el resto del archivo.

**Cursor**
- Configuración global: `~/.cursor/mcp.json`, dentro de la clave `mcpServers` en el nivel raíz. Aplica a todos los proyectos.
- Configuración de proyecto: `<proyecto>/.cursor/mcp.json` (fuera de alcance V1).
- Cursor tiene un límite práctico de ~40 tools activas en conjunto entre todos los servidores conectados; pasarse genera advertencia y algunas tools dejan de estar disponibles para el agente. Vale la pena reflejar esto en el panel de estado.

Ambos archivos usan el mismo formato estándar para cada entrada dentro de `mcpServers`:
```json
{
  "mcpServers": {
    "nombre-del-server": {
      "command": "npx",
      "args": ["-y", "algun-paquete"],
      "env": { "API_KEY": "valor" }
    }
  }
}
```
o, para servidores remotos:
```json
{
  "mcpServers": {
    "nombre-del-server": {
      "type": "http",
      "url": "https://mcp.ejemplo.com",
      "headers": { "Authorization": "Bearer TOKEN" }
    }
  }
}
```

## 5. Requerimientos funcionales

**RF-01 — Detección inicial (bootstrap)**
Al primer arranque, la app lee (solo lectura) `mcpServers` de `~/.claude.json` y de `~/.cursor/mcp.json`. Por cada entrada encontrada crea un registro en el catálogo local, marcado como `origen = auto_detectado` y `estado = habilitado` para ese cliente. Se muestra un resumen ("se detectaron N MCPs en Claude Code y M en Cursor").

**RF-02 — Catálogo local permanente**
Todo MCP detectado o creado manualmente vive indefinidamente en el catálogo, independientemente de si está activo. Cada registro guarda: nombre, tipo de transporte, definición completa (command/args/env o url/headers), en qué cliente(s) aplica, estado por cliente, origen, timestamps.

**RF-03 — Habilitar un MCP**
El usuario elige un MCP del catálogo y el/los cliente(s) destino. La app: (1) hace backup del archivo real, (2) inserta la entrada como último elemento del objeto `mcpServers`, (3) valida que el JSON resultante sea sintácticamente correcto, (4) escribe de forma atómica, (5) marca el registro como habilitado para ese cliente.

**RF-04 — Deshabilitar un MCP (soft delete)**
Igual que RF-03 pero en reversa: se quita la entrada del `mcpServers` real (con backup previo), y el registro queda intacto en el catálogo, marcado como deshabilitado para ese cliente. Al reabrir Claude Code o Cursor, ese MCP ya no carga ni consume recursos.

**RF-05 — Agregar un MCP nuevo manualmente**
Formulario para crear un registro desde cero (nombre, transporte, command/args/env o url/headers). Queda guardado en el catálogo con estado deshabilitado por defecto en ambos clientes, listo para habilitarse cuando se desee.

**RF-06 — Selección de cliente destino**
Toda acción de habilitar/deshabilitar debe permitir elegir explícitamente Claude Code, Cursor, o ambos — un mismo MCP puede estar activo en un cliente y no en el otro.

**RF-07 — Reconciliación de cambios externos**
Si el JSON real fue modificado por fuera de la app (edición manual, `claude mcp add`, instalación de un plugin, etc.), al abrir la app debe detectarse la diferencia entre catálogo y archivo real, y ofrecer importar el cambio o alertar del conflicto.

**RF-08 — Backup y rollback**
Cada escritura sobre un archivo real genera automáticamente un backup con timestamp. Debe existir una opción de restaurar el backup anterior si una escritura deja el archivo en estado inesperado.

**RF-09 — Panel de estado**
Vista principal con la lista completa del catálogo por cliente, indicador visual de habilitado/deshabilitado, y contador de "activos ahora" por cliente (ideal: estimado de tools que eso representa, dado el límite de Cursor mencionado arriba).

**RF-10 — Eliminación definitiva (hard delete)**
Acción separada de RF-04, con confirmación explícita, para borrar un registro del catálogo de forma permanente. Debe quedar claramente diferenciada de "deshabilitar" para evitar borrados accidentales.

## 6. Requerimientos no funcionales

| # | Requerimiento |
|---|---|
| RNF-01 | App nativa de macOS. Sugerido: Swift/SwiftUI (coherente con lo que ya veniste explorando en el proyecto del Nexgo N86). Alternativa: Electron/Tauri si en algún momento quieres portabilidad a Windows/Linux. |
| RNF-02 | Persistencia en SQLite embebido, no en un JSON plano propio — evita problemas de escritura concurrente y facilita filtros/consultas a futuro. |
| RNF-03 | Los `env`/`headers` reales pueden contener API keys en texto plano. La app no debe loguearlos ni exponerlos sin enmascarar en la UI. |
| RNF-04 | Escritura atómica (archivo temporal + rename) y validación de JSON antes de persistir — un trailing comma o llave mal cerrada puede dejar a Claude Code o Cursor sin cargar ningún MCP. |
| RNF-05 | V1 100% local/offline, sin dependencia de red. |
| RNF-06 | Claude Code y Cursor normalmente requieren reinicio de sesión para tomar cambios de config — la app debe avisarlo después de cada habilitar/deshabilitar. |
| RNF-07 | Operación prácticamente instantánea (los archivos son pequeños); no se requiere proceso en background salvo que se decida implementar un file-watcher (ver preguntas abiertas). |

## 7. Modelo de datos propuesto (catálogo local — SQLite)

**Tabla `mcp_catalog`**

| Campo | Tipo | Notas |
|---|---|---|
| id | uuid (PK) | |
| name | text | nombre del servidor |
| transport_type | enum(stdio, http, sse) | |
| command | text, nullable | solo stdio |
| args | json, nullable | solo stdio |
| env | json, nullable | variables de entorno |
| url | text, nullable | solo http/sse |
| headers | json, nullable | solo http/sse |
| source | enum(auto_detectado, manual) | |
| created_at | datetime | |
| updated_at | datetime | |

**Tabla `mcp_client_state`**

| Campo | Tipo | Notas |
|---|---|---|
| id | uuid (PK) | |
| mcp_id | uuid (FK → mcp_catalog.id) | |
| client | enum(claude_code, cursor) | |
| enabled | boolean | |
| last_enabled_at | datetime, nullable | |
| last_disabled_at | datetime, nullable | |

**Tabla `mcp_backups`**

| Campo | Tipo | Notas |
|---|---|---|
| id | uuid (PK) | |
| client | enum(claude_code, cursor) | |
| file_path | text | archivo real respaldado |
| backup_path | text | ubicación del backup |
| created_at | datetime | |
| reason | enum(pre_enable, pre_disable, manual) | |

## 8. Flujos principales

**Primer arranque**
1. Verifica existencia de `~/.claude.json` y `~/.cursor/mcp.json`.
2. Lee `mcpServers` de cada uno (solo lectura).
3. Crea/actualiza registros en `mcp_catalog` + `mcp_client_state` (enabled=true, source=auto_detectado).
4. Muestra resumen de lo detectado.

**Habilitar**
1. Usuario elige MCP + cliente(s).
2. Backup del archivo real.
3. Inserta la entrada al final de `mcpServers`.
4. Valida el JSON resultante.
5. Escribe de forma atómica.
6. Actualiza `mcp_client_state.enabled = true`.
7. Avisa que hay que reiniciar el cliente correspondiente.

**Deshabilitar**
Mismo flujo en reversa: backup → elimina la entrada de `mcpServers` → valida → escribe → `enabled = false` en el catálogo (el registro no se toca).

**Agregar nuevo**
1. Formulario con los campos de la tabla `mcp_catalog`.
2. Se guarda con `enabled = false` en ambos clientes.
3. Queda disponible para habilitar cuando se quiera.

## 9. Riesgos y consideraciones técnicas

- `~/.claude.json` mezcla MCPs con mucho más estado de la app (historial, onboarding, cuenta). Cualquier bug en la escritura que toque de más ese archivo puede romper la sesión de Claude Code — de ahí la importancia de RNF-04 y RF-08.
- Confundir el bloque `mcpServers` raíz (scope usuario) con el que vive dentro de `projects."<ruta>"` en el mismo archivo rompería el propósito del manager — son estructuras distintas dentro del mismo JSON.
- Si en algún momento agregas soporte a scope de proyecto, vas a necesitar saber "en qué proyecto estoy parado" para saber cuál `.mcp.json` / `.cursor/mcp.json` tocar — complejidad adicional real, no trivial.

## 10. Preguntas abiertas (decisiones que te faltan por tomar)

1. **Alcance de scope**: ¿V1 solo maneja configuración global (como se asumió aquí), o también quieres cubrir configuración por proyecto (`.mcp.json` / `.cursor/mcp.json` dentro de cada repo)?
2. **Stack tecnológico**: ¿Swift/SwiftUI nativo, o prefieres algo multiplataforma tipo Electron/Tauri pensando en portabilidad futura?
3. **Mismo nombre en ambos clientes**: si un MCP con el mismo nombre existe en Claude Code y Cursor pero con distinta configuración (ej. distinto token), ¿lo tratas como un solo registro "canónico" compartido, o como dos registros independientes que solo coinciden en el nombre visual?
4. **Detección de cambios**: ¿file-watcher en background que detecte drift en tiempo real, o te basta un botón manual de "refrescar / detectar cambios" (RF-07)?
5. **Hard delete**: ¿doble confirmación dado que es irreversible?

## 11. Fases sugeridas

- **Fase 1 (MVP)**: RF-01 a RF-04, RF-06, RF-08, RF-09. Catálogo SQLite, solo scope global, sin file-watcher.
- **Fase 2**: RF-05, RF-07, RF-10. Scope de proyecto, file-watcher, contador estimado de tools/contexto por cliente.
- **Fase 3**: soporte a otros clientes MCP (Claude Desktop, VS Code, Windsurf) reutilizando el mismo catálogo central.