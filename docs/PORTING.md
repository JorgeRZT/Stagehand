# Linux y Windows

El código es el mismo; solo cambia el empaquetado.

| Tema | macOS | Linux | Windows |
|---|---|---|---|
| Build | `cargo tauri build` → .app/.dmg | → .deb/.rpm/.AppImage | → .msi/.exe (NSIS) |
| Requisitos | Xcode CLT | `webkit2gtk-4.1`, `libayatana-appindicator` etc. | WebView2, MSVC |
| Abrir URL/carpeta | `open` | `xdg-open` | `cmd /C start` |
| Scripts AEM | `bin/start` | `bin/start` | `bin/start.bat` |
| Shell overrides | `sh -c` | `sh -c` | `cmd /C` |

Ya implementado con `cfg!` en `instance.rs` y `commands.rs::open_with_os`.

## Pendiente de validar en Windows/Linux
- `start.bat` puede quedarse en primer plano; si bloquea, usar `startCommand` con `start /B ...` o spawn desacoplado.
- Rutas con espacios en `startCommand`.
- Compilar cada SO en su propio runner (Tauri no cross-compila bien con WebView); ejemplo CI: GitHub Actions con `tauri-apps/tauri-action` en matriz macOS/ubuntu/windows.
- Notarización/firma macOS y firma Windows son pasos aparte.
