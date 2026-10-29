# AEM Environment Manager

App de escritorio (macOS primero; Linux y Windows con el mismo código) para **iniciar, detener y ver los logs** de instancias locales de **Adobe Experience Manager** (Author y Publish) sin usar la terminal.

- Lista de entornos con estado en vivo (Detenido / Iniciando / En ejecución / Deteniendo).
- Botones Iniciar / Detener / Abrir en navegador / Abrir carpeta.
- Visor de logs en tiempo real (`error.log`, `stdout.log`, `request.log`, …) con filtro de texto y de nivel.
- Configuración por entorno: puerto, runmodes extra, opciones JVM (p. ej. debug remoto) y comandos personalizados.

Stack: **Tauri 2** (backend Rust + UI HTML/JS sin build step). Binarios pequeños y multiplataforma.

## Requisitos

- Rust (`rustup`) y Tauri CLI: `cargo install tauri-cli --version "^2" --locked`
- Java instalado (el que necesite tu versión de AEM) para ejecutar las instancias.
- macOS: Xcode Command Line Tools (`xcode-select --install`).
- Linux: dependencias WebKitGTK (ver https://tauri.app/start/prerequisites/). Windows: WebView2 + MSVC.

## Uso en desarrollo

```bash
cargo tauri dev        # ejecuta la app con recarga de la UI
cargo tauri build      # genera .app/.dmg (macOS), .deb/.AppImage (Linux), .msi (Windows)
```

## Cómo se usa

Los entornos se agrupan en **workspaces** (por ejemplo, uno por cliente o proyecto): el selector de la parte superior del panel izquierdo cambia de workspace, ＋ crea uno y ✎ lo renombra o elimina. Cada workspace tiene sus propios Author y Publish.

1. Pulsa **＋** y sigue el asistente de 4 pasos con **Siguiente**: datos (nombre, workspace, tipo: Author 4502 / Publish 4503), carpeta del entorno y JAR (si la carpeta contiene un `.jar` se usa para arrancar; si hay varios eliges cuál es el de AEM; si no hay ninguno indicas dónde está y se copia a esa carpeta), y por último puerto, runmodes y opciones JVM, más comandos personalizados opcionales.
2. Selecciona el entorno y pulsa **Iniciar**. El estado pasa a *En ejecución* cuando el puerto responde.
3. Elige el log en el desplegable; se actualiza cada segundo. Usa los filtros para buscar texto o nivel.
4. **Detener** ejecuta `crx-quickstart/bin/stop`.

La configuración se guarda en `~/Library/Application Support/com.aemenv.manager/environments.json` (macOS), `~/.config/com.aemenv.manager/` (Linux) o `%APPDATA%\com.aemenv.manager\` (Windows).

## Documentación

| Archivo | Contenido |
|---|---|
| [MEMORY.md](MEMORY.md) | Contexto compacto para LLMs: decisiones, convenciones, estado, gotchas. **Leer primero.** |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Módulos, flujo de datos, API backend↔UI |
| [docs/EXTENDING.md](docs/EXTENDING.md) | Recetas paso a paso para añadir funcionalidades |
| [docs/PORTING.md](docs/PORTING.md) | Linux / Windows: qué cambia y cómo compilar |

## Limitaciones conocidas

- No hay tests automáticos aún; verificado manualmente.
- "Iniciando" termina cuando el puerto HTTP acepta conexiones, aunque AEM aún cargue bundles.
- El estado se detecta por puerto: una instancia arrancada fuera de la app también aparece como *En ejecución*.
