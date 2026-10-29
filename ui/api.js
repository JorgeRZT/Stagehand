// Thin wrapper over the Tauri backend. This is the ONLY file that knows about `invoke`.
// Every backend command in src-tauri/src/commands.rs has a counterpart here.
// To run the UI in a plain browser (no Tauri), the fallback below logs and returns empty data.
const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : async (cmd) => { console.warn('no tauri:', cmd); return null; };

const api = {
  listWorkspaces: () => invoke('list_workspaces'),             // -> [{ id, name }] (never empty)
  saveWorkspace: (workspace) => invoke('save_workspace', { workspace }),
  deleteWorkspace: (id) => invoke('delete_workspace', { id }), // also removes its environment entries
  listEnvironments: () => invoke('list_environments'),
  scanJars: (dir) => invoke('scan_jars', { dir }),             // -> file names of *.jar in dir
  saveEnvironment: (env) => invoke('save_environment', { env }),
  deleteEnvironment: (id) => invoke('delete_environment', { id }),
  getStatus: (id) => invoke('get_status', { id }),            // 'stopped' | 'starting' | 'running' | 'stopping'
  start: (id) => invoke('start_environment', { id }),
  stop: (id) => invoke('stop_environment', { id }),
  listLogs: (id) => invoke('list_logs', { id }),
  readLog: (id, file, offset) => invoke('read_log', { id, file, offset }), // -> { text, offset }
  openUrl: (url) => invoke('open_url', { url }),
  revealPath: (path) => invoke('reveal_path', { path }),
  // window.confirm/alert/prompt are unreliable in the macOS WebView: use the dialog plugin.
  confirm: (message) => window.__TAURI__.dialog.ask(message, { kind: 'warning' }),
  alert: (message) => window.__TAURI__.dialog.message(message, { kind: 'error' }),
  pickJar: async () => window.__TAURI__.dialog.open({ multiple: false, filters: [{ name: 'JAR', extensions: ['jar'] }] }),
  pickDirectory: async () => window.__TAURI__.dialog.open({ directory: true, multiple: false }),
};
