# MEMORY.md – contexto para LLMs / futuros colaboradores

> Lee este archivo antes de tocar el código. Mantenlo actualizado al terminar cada cambio relevante (sección "Estado" y "Decisiones").

## Qué es
App de escritorio para gestionar instancias AEM locales (Author/Publish): start, stop, logs. Objetivo: que los equipos no usen terminal. Primero macOS, portable a Linux/Windows.

## Stack y por qué
- **Tauri 2** (Rust + WebView del SO). Elegido sobre Electron por tamaño y multiplataforma sin reescribir. Sin Node en runtime.
- **UI = HTML/CSS/JS plano en `ui/`**, sin framework ni bundler (`withGlobalTauri: true`). Decisión consciente: cero paso de build, fácil de editar por LLMs. Si crece mucho, migrar a Vite + framework y cambiar `frontendDist` en `tauri.conf.json`.

## Mapa de archivos
- `src-tauri/src/lib.rs` – registro de módulos y comandos (`generate_handler!`).
- `src-tauri/src/config.rs` – modelo `Environment` + persistencia JSON.
- `src-tauri/src/instance.rs` – start/stop/status (ÚNICO lugar que lanza procesos).
- `src-tauri/src/logs.rs` – listar y leer logs incrementalmente por offset.
- `src-tauri/src/commands.rs` – capa fina `#[tauri::command]`.
- `ui/api.js` – ÚNICO sitio que usa `invoke`. `ui/app.js` – lógica UI. `ui/styles.css`, `ui/index.html`.
- `docs/` – ARCHITECTURE, EXTENDING, PORTING.

## Decisiones clave
- Arranque: orden `startCommand` → **JAR configurado (`jar`, nombre de archivo dentro de `dir`) con `java -jar … -r <runmodes> -p <port>`** → `crx-quickstart/bin/start`. Parada: `stopCommand` → `crx-quickstart/bin/stop`.
- Al crear un entorno la UI escanea `*.jar` de la carpeta (`scan_jars`): 1 → auto, varios → el usuario elige, 0 → se pide la ruta y `save_environment` lo **copia a `dir`** (async) guardando solo el nombre.
- Estado = puerto TCP abierto en 127.0.0.1 + transición en memoria (`Transitions`). No guardamos PIDs; así también detecta instancias lanzadas fuera de la app.
- Logs por **polling con offset** (UI cada 1 s) en vez de watchers: igual en todos los SO, soporta rotación.
- `read_log` rechaza nombres con `/`, `\`, `..` (path traversal). `open_url` solo http/https.
- **Workspaces**: `Workspace{id,name}` en `workspaces.json`; cada `Environment` tiene `workspaceId`. Entornos antiguos sin campo → workspace `default` (siempre existe al menos uno). La UI guarda todos los entornos en memoria y filtra por `currentWs` (último elegido en localStorage). Borrar un workspace borra sus entradas de entornos, no archivos.
- El formulario de entorno es un **asistente por pasos** (Datos → Carpeta y JAR → Ejecución → Comandos): un `<fieldset class="step">` por paso en `ui/index.html`, lógica en `ui/app.js` (`showStep`, `validateStep`, `goNext`). Validación por paso (el `<form>` es `novalidate`); Enter avanza. Al editar se puede saltar entre pasos.
- `confirm/alert/prompt` del WebView no son fiables en macOS: usar `api.confirm/api.alert` (plugin dialog).
- Config en el directorio de config de la app (`app_config_dir`), `environments.json`.

## Convenciones
- Comando nuevo = función en `commands.rs` + registro en `lib.rs` + wrapper en `ui/api.js`.
- Errores del backend son `String` y la UI los muestra en el banner `#msg`.
- No añadir lógica de procesos fuera de `instance.rs`.
- Código específico de SO con `cfg!`/`#[cfg(...)]` y documentarlo en `docs/PORTING.md`.

## Estado
- [x] CRUD de entornos, start/stop/status, visor de logs con filtros, abrir navegador/carpeta.
- [ ] Probado contra una instancia AEM real (se verificó compilación; el autor no tenía quickstart a mano).
- [ ] Probado en Linux/Windows (código preparado, no ejecutado).
- [x] Test unitario de `logs` (`cd src-tauri && cargo test`). Falta cobertura de `instance`/`config`.
- [x] `cargo tauri build --bundles app` genera el .app en `src-tauri/target/release/bundle/macos/`.
- Icono: fuente `app-icon.png`; regenerar con `cargo tauri icon app-icon.png`.

## Ideas / siguientes pasos
Importar entornos escaneando carpetas; perfiles de memoria; ver estado de bundles vía `/system/console`; bandeja del sistema; notificación al terminar de arrancar; empaquetar y firmar (notarización macOS); auto-update.

## Gotchas
- Si `bin/stop` falla (típico: instancia lanzada con `java -jar`, sin `cq.pid`; salida «cq.pid not found / Unable to access jarfile stop»), `instance::stop` hace fallback: SIGTERM (`kill -TERM`/`taskkill`) al proceso que escucha en el puerto (`lsof`/`netstat`). Ojo: mata cualquier proceso que use ese puerto.
- `stop` espera (máx. 30 s) a que termine `bin/stop` y devuelve su salida si falla (exit ≠ 0); antes lo descartaba y la UI se quedaba en «Deteniendo…». El estado «Deteniendo» solo acaba cuando el puerto se cierra (AEM puede tardar minutos); el botón Detener sigue activo para reintentar.
- Los scripts `bin/start|stop` desempaquetados pueden no tener el bit de ejecución (error `Permission denied (os error 13)`): en Unix se ejecutan con `bash <script>` (`instance::script_command`), nunca directamente.
- `bin/start` daemoniza: el proceso hijo termina enseguida, por eso no se guarda PID.
- En Windows `start.bat` no demoniza igual; revisar `docs/PORTING.md`.
- Con `withGlobalTauri` la API está en `window.__TAURI__` (no hay imports ES de `@tauri-apps/api`).
