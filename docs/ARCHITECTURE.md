# Architecture

```
┌────────────── ui/ (WebView) ──────────────┐        ┌──────────── src-tauri/ (Rust) ───────────┐
│ index.html  styles.css  app.js  api.js    │ invoke │ commands.rs ─┬─ config.rs  (JSON on disk) │
│  app.js → api.js ───────────────────────────────────▶            ├─ instance.rs (processes)   │
│  polling: status 2 s, logs 1 s            │◀────── │              └─ logs.rs     (reading)     │
└───────────────────────────────────────────┘ result └────────────────────────────────────────────┘
```

## Models (config.rs ↔ ui/app.js)
- `Workspace`: `id, name`.
- `Environment`: `id, workspaceId, name, kind (author|publish), dir, jar, port, extraRunmodes, jvmOpts, startCommand, stopCommand` (camelCase in JSON). A missing `workspaceId` defaults to `default`.

## Backend API (commands.rs → ui/api.js)
| Command | Input | Output |
|---|---|---|
| `list_workspaces` | – | `Workspace[]` (never empty) |
| `save_workspace` | `workspace` | – (upsert) |
| `delete_workspace` | `id` | – (also removes its environments from the list) |
| `list_environments` | – | `Environment[]` (all workspaces) |
| `scan_jars` | `dir` | `*.jar` file names in the folder |
| `save_environment` | `env` | – (upsert by id; if `env.jar` is an absolute path it is copied into `dir`) |
| `delete_environment` | `id` | – (does not delete files) |
| `get_status` | `id` | `stopped\|starting\|running\|stopping` |
| `start_environment` / `stop_environment` | `id` | – (error string on failure) |
| `list_logs` | `id` | `*.log` names in `crx-quickstart/logs` |
| `read_log` | `id, file, offset?` | `{text, offset}`; no offset = last 64 KB |
| `open_url` | `url` | – (http/https only) |
| `reveal_path` | `path` | – (opens in the file manager) |

## Instance status
`instance::status`: port open → `running`; a `Starting` transition with the port still closed → `starting`; `Stopping` with the port open → `stopping`. Transitions expire after 15 min.

## Start
Order: `startCommand` → `java -jar <jar> -r <runmodes> -p <port>` (configured JAR) → `crx-quickstart/bin/start`. The env vars `CQ_PORT`, `CQ_RUNMODE`, `CQ_JVM_OPTS` are always injected.

## Stop
Order: `stopCommand` → `crx-quickstart/bin/stop` (waits up to 30 s; non-zero exit is reported). If the script fails or is missing and no custom command is set, the process listening on the port receives SIGTERM / `taskkill`.

## Logs
Each call returns up to 256 KB from `offset`. The UI splits into lines, keeps the trailing partial line, trims to ~400k characters and filters client-side (level detected from `*ERROR*`, `*WARN*`…; lines without a level, such as stack traces, are always shown).
