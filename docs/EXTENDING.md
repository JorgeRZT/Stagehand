# How to extend (recipes)

## Add a backend command
1. Function in `src-tauri/src/commands.rs` with `#[tauri::command]` (load the environment with `config::find`, delegate to a module).
2. Register it in `generate_handler![...]` in `lib.rs`.
3. Wrapper in `ui/api.js` (argument names are camelCase in JS and snake_case in Rust; Tauri converts them).
4. Document it in the table in `docs/ARCHITECTURE.md`.

## Add a field to an environment
1. `config.rs`: add the field to `Environment` with `#[serde(default)]` (does not break old configs).
2. `ui/index.html`: add an `<input name="field">` inside the `<form>`, in the `<fieldset class="step">` it belongs to (or create a new step and add its `<li>` to `#steps`).
3. `ui/app.js`: add it to the default object in `openDialog` and to the `env` object in the submit handler.
4. Use it in `instance.rs`.

## Concrete examples
- **Bundle status / health check**: in `instance.rs` GET `http://localhost:{port}/system/console/bundles.json` (add the `ureq` crate) to tell "Starting" from "Ready".
- **Import environments**: a command that walks a folder looking for `crx-quickstart` and returns candidates; a button in the sidebar.
- **New log colors**: `ui/app.js::renderLog` (switch to HTML with spans, escaping the text).
- **Notifications**: `tauri-plugin-notification`, add the permission in `capabilities/default.json`.
- **Tests**: pure logic (`runmodes`, `logs::read`) is testable with `cargo test` in `src-tauri`.

## Rules
- Processes only in `instance.rs`; `invoke` only in `api.js`.
- Everything in English (code, comments, UI text, docs).
- Update `MEMORY.md` (Status / Key decisions) when done.
- Verify with `cargo check --manifest-path src-tauri/Cargo.toml` and `cargo tauri dev`.
