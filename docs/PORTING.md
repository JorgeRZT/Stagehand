# Linux and Windows

The code is the same; only packaging changes.

| Topic | macOS | Linux | Windows |
|---|---|---|---|
| Build | `cargo tauri build` → .app/.dmg | → .deb/.rpm/.AppImage | → .msi/.exe (NSIS) |
| Requirements | Xcode CLT | `webkit2gtk-4.1`, `libayatana-appindicator`, etc. | WebView2, MSVC |
| Open URL/folder | `open` | `xdg-open` | `cmd /C start` |
| AEM scripts | `bash bin/start` | `bash bin/start` | `bin/start.bat` |
| Command overrides | `sh -c` | `sh -c` | `cmd /C` |
| Find process on port | `lsof` | `lsof` | `netstat -ano` + `taskkill` |

Already implemented with `cfg!` / `#[cfg]` in `instance.rs` and `commands.rs::open_with_os`.

## Still to validate on Windows/Linux
- `start.bat` may stay in the foreground; if it blocks, use `startCommand` with `start /B ...` or a detached spawn.
- Paths with spaces in `startCommand`.
- Build each OS on its own runner (Tauri does not cross-compile well with WebViews); example CI: GitHub Actions with `tauri-apps/tauri-action` on a macOS/ubuntu/windows matrix.
- macOS notarization/signing and Windows signing are separate steps.
