# Stagehand

> Local environment manager for Adobe Experience Manager.

![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white)
![JavaScript](https://img.shields.io/badge/UI-vanilla%20JS-F7DF1E?logo=javascript&logoColor=black)
![Adobe Experience Manager](https://img.shields.io/badge/Adobe-AEM-EB1000?logo=adobe&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-supported-000000?logo=apple&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-untested-FCC624?logo=linux&logoColor=black)
![Windows](https://img.shields.io/badge/Windows-untested-0078D6?logo=windows&logoColor=white)
![LLM friendly](https://img.shields.io/badge/docs-LLM%20friendly-8A2BE2)

**Stagehand** is a desktop app (macOS first; Linux and Windows from the same code) to **start, stop and read the logs** of local **Adobe Experience Manager** instances (Author and Publish) without using the terminal.

- Workspaces to group environments (e.g. one per client or project).
- Environment list with live status (Stopped / Starting / Running / Stopping).
- Start / Stop / Open in browser / Open folder buttons.
- Real-time log viewer (`error.log`, `stdout.log`, `request.log`, …) with text and level filters.
- Per-environment settings: port, extra runmodes, JVM options (e.g. remote debug) and custom commands.

Stack: **Tauri 2** (Rust backend + plain HTML/JS UI with no build step). Small, cross-platform binaries.

## Requirements

- Rust (`rustup`) and the Tauri CLI: `cargo install tauri-cli --version "^2" --locked`
- Java installed (the version your AEM release needs) to run the instances.
- macOS: Xcode Command Line Tools (`xcode-select --install`).
- Linux: WebKitGTK dependencies (see https://tauri.app/start/prerequisites/). Windows: WebView2 + MSVC.

## Development

```bash
cargo tauri dev        # run the app with UI reload
cargo tauri build      # produce .app/.dmg (macOS), .deb/.AppImage (Linux), .msi (Windows)
cd src-tauri && cargo test
```

## Usage

Environments are grouped in **workspaces** (for example one per client or project): the selector at the top of the left panel switches workspace, ＋ creates one and ✎ renames or deletes it. Each workspace has its own Author and Publish environments.

1. Press **＋** and follow the 4-step wizard with **Next**:
   1. Details: name, workspace, type (Author 4502 / Publish 4503).
   2. Folder and JAR: if the folder contains a `.jar` it is used to start the instance; if there are several you choose which one is AEM; if there is none you point to it and it is copied into the folder.
   3. Runtime: port, extra runmodes and JVM options.
   4. Optional custom start/stop commands.
2. Select the environment and press **Start**. The status becomes *Running* once the port answers.
3. Pick a log in the dropdown; it refreshes every second. Use the filters to search by text or level.
4. **Stop** runs `crx-quickstart/bin/stop`; if that fails it asks the process listening on the port to exit.

Configuration is stored in `~/Library/Application Support/com.stagehand.app/` (macOS), `~/.config/com.stagehand.app/` (Linux) or `%APPDATA%\com.stagehand.app\` (Windows), as `environments.json` and `workspaces.json`.

## Documentation

| File | Content |
|---|---|
| [ROADMAP.md](ROADMAP.md) | Planned work; next step: local Dispatcher support |
| [MEMORY.md](MEMORY.md) | Compact context for LLMs: decisions, conventions, status, gotchas. **Read first.** |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Modules, data flow, backend ↔ UI API |
| [docs/EXTENDING.md](docs/EXTENDING.md) | Step-by-step recipes to add features |
| [docs/PORTING.md](docs/PORTING.md) | Linux / Windows: what changes and how to build |

All code and documentation in this repository is written in English.

## Known limitations

- Automated tests only cover log reading.
- "Starting" ends when the HTTP port accepts connections, even if AEM is still loading bundles.
- Status is detected by port: an instance started outside the app also shows as *Running*.
