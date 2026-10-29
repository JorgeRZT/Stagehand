# MEMORY.md – Stagehand: context for LLMs / future contributors

> Read this file before touching the code. Keep it up to date after every relevant change (sections "Status" and "Key decisions").

## Language policy
**Everything in this repo is written in English**: README, MEMORY.md, docs, code, comments, UI text, error messages, commit messages, issues/PRs. Do not add Spanish (or any other language) to files. Chat with the maintainer may happen in any language.

## What it is
Stagehand (formerly "AEM Environment Manager", GitHub repo renamed accordingly) is a desktop app to manage local AEM instances (Author/Publish): start, stop, logs. Goal: development teams should not need a terminal. macOS first, portable to Linux/Windows.

## Stack and why
- **Tauri 2** (Rust + the OS WebView). Chosen over Electron for size and cross-platform builds without rewrites. No Node at runtime.
- **UI = plain HTML/CSS/JS in `ui/`**, no framework or bundler (`withGlobalTauri: true`). Deliberate decision: zero build step, easy for LLMs to edit. If it grows a lot, migrate to Vite + a framework and change `frontendDist` in `tauri.conf.json`.

## File map
- `src-tauri/src/lib.rs` – module and command registration (`generate_handler!`).
- `src-tauri/src/config.rs` – `Environment` / `Workspace` models + JSON persistence.
- `src-tauri/src/instance.rs` – start/stop/status (the ONLY place that spawns processes).
- `src-tauri/src/logs.rs` – list logs and read them incrementally by offset.
- `src-tauri/src/commands.rs` – thin `#[tauri::command]` layer.
- `ui/api.js` – the ONLY place that uses `invoke`. `ui/app.js` – UI logic. `ui/styles.css`, `ui/index.html`.
- `docs/` – ARCHITECTURE, EXTENDING, PORTING.

## Key decisions
- Start order: `startCommand` → **configured JAR (`jar`, a file name inside `dir`) via `java -jar … -r <runmodes> -p <port>`** → `crx-quickstart/bin/start`. Stop: `stopCommand` → `crx-quickstart/bin/stop` → fallback (see Gotchas).
- When creating an environment the UI scans the folder for `*.jar` (`scan_jars`): 1 → auto-selected, several → the user chooses, 0 → the user is asked for the path and `save_environment` **copies the jar into `dir`** (async) and stores only the file name.
- Status = TCP port open on 127.0.0.1 + in-memory transition (`Transitions`). PIDs are not stored, so instances started outside the app are detected too.
- Logs use **polling with an offset** (UI every 1 s) instead of file watchers: identical on every OS and handles rotation.
- `read_log` rejects names containing `/`, `\`, `..` (path traversal). `open_url` only allows http/https.
- **Workspaces**: `Workspace{id,name}` in `workspaces.json`; every `Environment` has a `workspaceId`. Old environments without the field → workspace `default` (at least one workspace always exists). The UI keeps all environments in memory and filters by `currentWs` (last choice stored in localStorage). Deleting a workspace removes its environment entries, never files.
- The environment form is a **step-by-step wizard** (Details → Folder & JAR → Runtime → Commands): one `<fieldset class="step">` per step in `ui/index.html`, logic in `ui/app.js` (`showStep`, `validateStep`, `goNext`). Per-step validation (the `<form>` is `novalidate`); Enter advances. When editing, any step can be jumped to.
- The sidebar list shows an `A` (Author, blue) / `P` (Publish, purple) icon before each environment name (`.kind-icon`).
- `confirm/alert/prompt` of the WebView are unreliable on macOS: use `api.confirm` / `api.alert` (dialog plugin).
- Config lives in the app config dir (`app_config_dir`, identifier `com.stagehand.app`): `environments.json` and `workspaces.json`. On first run `config::config_dir` copies them from the legacy `com.aemenv.manager` dir if present.

## Releases
- Version = calendar date. Installers are NOT committed: they are published as GitHub Releases (`v<version>`). Local builds go to the git-ignored `dist/<YYYY.MM.DD>/` (zero-padded folder). The app version in `tauri.conf.json`/`Cargo.toml` is the SemVer-safe form without leading zeros (`2026.10.29`, but `2026.3.5` for March 5th). `scripts/release.sh` does all of it; `--publish` also creates the GitHub Release `v<version>` with the DMG attached (commit and push first; the DMG from `dist/` is uploaded as the release asset). The README "Download" section links to the latest release and to the direct asset URL; update the direct link when releasing a new version.
- Builds are unsigned/not notarized (macOS Gatekeeper warns on first launch). Only the host architecture is built (currently aarch64).

## Documentation assets
- README screenshots live in `docs/screenshots/` (running instance + logs, workspace/environments, creation wizard, log picker). Retake them when the UI changes noticeably.

## Conventions
- English only (see Language policy).
- New command = function in `commands.rs` + registration in `lib.rs` + wrapper in `ui/api.js`.
- Backend errors are `String`s and the UI shows them in the `#msg` banner.
- No process-spawning logic outside `instance.rs`.
- OS-specific code via `cfg!` / `#[cfg(...)]`, documented in `docs/PORTING.md`.

## Status
- [x] Environment CRUD, workspaces, start/stop/status, log viewer with filters, open browser/folder.
- [ ] Tested against a real AEM instance end to end (compilation and unit test verified; stop fallback added after a real failure report).
- [ ] Tested on Linux/Windows (code prepared, never executed).
- [x] Unit test for `logs` (`cd src-tauri && cargo test`). `instance`/`config` have no coverage yet.
- [x] `cargo tauri build --bundles app` produces the .app in `src-tauri/target/release/bundle/macos/`.
- Icon: source is `app-icon.png`; regenerate with `cargo tauri icon app-icon.png`.

## Ideas / next steps
**Next planned step: local Dispatcher support** (new `dispatcher` environment kind) — see [ROADMAP.md](ROADMAP.md).

Other ideas: Import environments by scanning folders; memory profiles; bundle status via `/system/console`; system tray; notification when startup finishes; packaging and signing (macOS notarization); auto-update.

## Gotchas
- If `bin/stop` fails (typical: instance started with `java -jar`, so no `cq.pid`; output like "cq.pid not found / Unable to access jarfile stop"), `instance::stop` falls back to SIGTERM (`kill -TERM` / `taskkill`) on the process listening on the port (`lsof` / `netstat`). Beware: it signals any process using that port.
- `stop` waits (max 30 s) for `bin/stop` to exit and returns its output if it fails (exit ≠ 0); it used to discard it and the UI stayed in "Stopping…". "Stopping" only ends when the port closes (AEM can take minutes); the Stop button stays enabled to retry.
- Unpacked `bin/start|stop` scripts may lack the executable bit (`Permission denied (os error 13)`): on Unix they are run as `bash <script>` (`instance::script_command`), never directly.
- `bin/start` daemonizes: the child exits quickly, which is why no PID is stored.
- On Windows `start.bat` does not daemonize the same way; see `docs/PORTING.md`.
- With `withGlobalTauri` the API is at `window.__TAURI__` (no ES imports from `@tauri-apps/api`).
