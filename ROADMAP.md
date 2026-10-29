# Roadmap

Planned work, in priority order. Update this file (and the "Status" / "Ideas" sections of [MEMORY.md](MEMORY.md)) whenever an item starts or ships.

## Next: local Dispatcher support

**Goal:** besides Author and Publish, manage a **local AEM Dispatcher** from the same app, so a team can run the full Author → Publish → Dispatcher chain per workspace without touching the terminal.

The details below are a proposal, not a design decision. Confirm them before implementing.

### Scope
- New environment kind `dispatcher` next to `author` and `publish` (with its own `D` icon in the sidebar list).
- Start / stop / status for the Dispatcher, using the same lifecycle model as AEM instances.
- Logs of the web server and the Dispatcher module in the existing log viewer (e.g. `httpd` access/error logs and the Dispatcher module log).
- Link a Dispatcher to the Publish environment of the same workspace it fronts (render backend host and port).
- Per-environment settings: HTTP port, path to the Dispatcher configuration (the `dispatcher-sdk` folder or an `httpd`/`conf.d`/`conf.dispatcher.d` layout), and optional extra arguments.

### Open questions
- Runtime: the AEM Dispatcher SDK for local use (which ships a Docker-based `bin/docker_run`-style workflow) versus a native Apache httpd with the Dispatcher module. The SDK path is the likelier default.
- Whether Docker should be a hard requirement, and how to detect and report that it is missing.
- How status is detected (port check as today, or container state).
- Validating the Dispatcher configuration before starting, and surfacing validation errors in the UI.
- Cache folder (docroot) handling: show its location and offer a "clear cache" action.

### Likely code impact
- `config.rs`: add `Kind::Dispatcher` and the new fields with `#[serde(default)]`.
- `instance.rs`: Dispatcher start/stop strategies (still the only place that spawns processes).
- `logs.rs`: log directory resolution that is not tied to `crx-quickstart/logs`.
- `ui/`: wizard step(s) for Dispatcher settings and the `D` icon, following [docs/EXTENDING.md](docs/EXTENDING.md).
- Docs: update `docs/ARCHITECTURE.md`, `docs/PORTING.md` and `MEMORY.md` when it lands.

## Later
- Import existing environments by scanning folders.
- Health check via `/system/console/bundles.json` to tell "Starting" from "Ready".
- Notification when startup finishes; system tray.
- Memory/JVM profiles.
- Validate on Linux and Windows; CI builds for all three platforms.
- Packaging and signing (macOS notarization), auto-update.
- Test coverage for `instance` and `config`.
