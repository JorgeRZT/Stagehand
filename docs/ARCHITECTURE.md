# Arquitectura

```
┌────────────── ui/ (WebView) ──────────────┐        ┌──────────── src-tauri/ (Rust) ───────────┐
│ index.html  styles.css  app.js  api.js    │ invoke │ commands.rs ─┬─ config.rs  (JSON en disco)│
│  app.js → api.js ───────────────────────────────────▶            ├─ instance.rs (procesos)    │
│  polling: estado 2 s, logs 1 s            │◀────── │              └─ logs.rs     (lectura)     │
└───────────────────────────────────────────┘ result └────────────────────────────────────────────┘
```

## Modelo `Environment` (config.rs ↔ ui/app.js)
`id, workspaceId, name, kind (author|publish), dir, jar, port, extraRunmodes, jvmOpts, startCommand, stopCommand` (camelCase en JSON).

## API backend (commands.rs → ui/api.js)
| Comando | Entrada | Salida |
|---|---|---|
| `list_workspaces` | – | `Workspace[]` (`{id,name}`, nunca vacío) |
| `save_workspace` | `workspace` | – (upsert) |
| `delete_workspace` | `id` | – (elimina también sus entornos de la lista) |
| `list_environments` | – | `Environment[]` (todos los workspaces) |
| `scan_jars` | `dir` | nombres `*.jar` de la carpeta |
| `save_environment` | `env` | – (upsert por id; si `env.jar` es ruta absoluta la copia a `dir`) |
| `delete_environment` | `id` | – (no borra archivos) |
| `get_status` | `id` | `stopped\|starting\|running\|stopping` |
| `start_environment` / `stop_environment` | `id` | – (error string si falla) |
| `list_logs` | `id` | nombres `*.log` de `crx-quickstart/logs` |
| `read_log` | `id, file, offset?` | `{text, offset}`; sin offset = últimos 64 KB |
| `open_url` | `url` | – (solo http/https) |
| `reveal_path` | `path` | – (abre en el gestor de archivos) |

## Estado de una instancia
`instance::status`: puerto abierto → `running`; si hay transición `Starting` y el puerto aún cerrado → `starting`; `Stopping` y puerto abierto → `stopping`. Las transiciones caducan a los 15 min.

## Arranque
Orden: `startCommand` → `java -jar <jar> -r <runmodes> -p <port>` (JAR configurado) → `crx-quickstart/bin/start`. Variables de entorno `CQ_PORT`, `CQ_RUNMODE`, `CQ_JVM_OPTS` siempre se inyectan.

## Logs
Cada llamada devuelve hasta 256 KB desde `offset`. La UI parte por líneas, conserva la línea parcial final, recorta a ~400k caracteres y filtra en cliente (el nivel se detecta con `*ERROR*`, `*WARN*`…; las líneas sin nivel, como stack traces, siempre se muestran).
