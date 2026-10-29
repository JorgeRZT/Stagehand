# Cómo extender (recetas)

## Añadir un comando backend
1. Función en `src-tauri/src/commands.rs` con `#[tauri::command]` (cargar entorno con `config::find`, delegar en un módulo).
2. Registrarla en `generate_handler![...]` de `lib.rs`.
3. Wrapper en `ui/api.js` (nombres de argumentos en camelCase en JS, snake_case en Rust se convierte automáticamente).
4. Documentarla en la tabla de `docs/ARCHITECTURE.md`.

## Añadir un campo a un entorno
1. `config.rs`: campo en `Environment` con `#[serde(default)]` (no rompe configs antiguas).
2. `ui/index.html`: `<input name="campo">` dentro del `<form>`.
3. Colócalo en el `<fieldset class="step">` que corresponda (o crea uno nuevo y añade su `<li>` en `#steps`). `ui/app.js`: añadirlo al objeto por defecto de `openDialog` y al objeto `env` del submit.
4. Usarlo en `instance.rs`.

## Ejemplos concretos
- **Estado de bundles / healthcheck**: en `instance.rs` hacer GET a `http://localhost:{port}/system/console/bundles.json` (añadir crate `ureq`) y distinguir "Iniciando" de "Listo".
- **Importar entornos**: comando que recorra una carpeta buscando `crx-quickstart` y devuelva candidatos; botón en el sidebar.
- **Nuevo tipo de log / colores**: `ui/app.js::renderLog` (cambiar a HTML con spans escapando el texto).
- **Notificaciones**: plugin `tauri-plugin-notification`, añadir permiso en `capabilities/default.json`.
- **Tests**: la lógica pura (`runmodes`, `logs::read`) es testeable con `cargo test` en `src-tauri`.

## Reglas
- Procesos solo en `instance.rs`; invoke solo en `api.js`.
- Actualizar `MEMORY.md` (Estado/Decisiones) al terminar.
- Verificar con `cargo check --manifest-path src-tauri/Cargo.toml` y `cargo tauri dev`.
