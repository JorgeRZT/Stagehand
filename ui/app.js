// UI logic. State is kept in a few module-level variables; rendering is imperative and small.
const $ = (id) => document.getElementById(id);
const STATE_LABEL = { stopped: 'Detenido', starting: 'Iniciando…', running: 'En ejecución', stopping: 'Deteniendo…' };
const MAX_LOG_CHARS = 400_000;

let workspaces = [];
let currentWs = null;    // id of the workspace being shown
let envs = [];           // ALL environments (every workspace)
let statuses = {};       // id -> state
let selectedId = null;
let log = { id: null, file: null, offset: null, lines: [] };
let editing = null;      // env being edited (null = new)

const visibleEnvs = () => envs.filter((e) => e.workspaceId === currentWs);
const selected = () => visibleEnvs().find((e) => e.id === selectedId);
const safeStore = { get: (k) => { try { return localStorage.getItem(k); } catch { return null; } }, set: (k, v) => { try { localStorage.setItem(k, v); } catch { /* ignore */ } } };

function showMsg(text) {
  const m = $('msg');
  m.textContent = text || '';
  m.hidden = !text;
}
const guard = (fn) => async (...a) => { try { showMsg(''); await fn(...a); } catch (e) { showMsg(String(e)); } };

// ---------- environments list ----------
async function refreshEnvs() {
  workspaces = (await api.listWorkspaces()) || [];
  envs = (await api.listEnvironments()) || [];
  if (!workspaces.find((w) => w.id === currentWs)) {
    const last = safeStore.get('workspace');
    currentWs = (workspaces.find((w) => w.id === last) || workspaces[0])?.id ?? null;
  }
  renderWorkspaces();
  if (!selected()) selectedId = visibleEnvs()[0]?.id ?? null;
  log.id = null; // force log reload (workspace or dir may have changed)
  renderList();
  renderDetail();
}

function renderWorkspaces() {
  $('ws-select').replaceChildren(...workspaces.map((w) => new Option(w.name, w.id)));
  $('ws-select').value = currentWs;
}

function renderList() {
  const list = visibleEnvs();
  $('empty').hidden = list.length > 0;
  $('env-list').replaceChildren(...list.map((e) => {
    const li = document.createElement('li');
    li.className = e.id === selectedId ? 'active' : '';
    li.innerHTML = `<span class="dot ${statuses[e.id] || 'stopped'}"></span><span class="n"></span><small>${e.kind} :${e.port}</small>`;
    li.querySelector('.n').textContent = e.name;
    li.onclick = () => select(e.id);
    return li;
  }));
}

function select(id) {
  selectedId = id;
  renderList();
  renderDetail();
}

async function renderDetail() {
  const env = selected();
  $('detail').hidden = !env;
  $('placeholder').hidden = !!env;
  if (!env) { log.id = null; return; }
  $('env-name').textContent = env.name;
  $('badge').textContent = env.kind;
  $('badge').className = `badge ${env.kind}`;
  renderState();
  if (log.id !== env.id) await resetLog(env);
}

function renderState() {
  const env = selected();
  if (!env) return;
  const s = statuses[env.id] || 'stopped';
  $('env-state').textContent = STATE_LABEL[s];
  $('env-state').className = `state ${s}`;
  $('btn-start').disabled = s !== 'stopped';
  $('btn-stop').disabled = s === 'stopped'; // also enabled while 'stopping' so it can be retried
  $('btn-open').disabled = s !== 'running';
}

async function pollStatuses() {
  for (const e of envs) {
    try { statuses[e.id] = await api.getStatus(e.id); } catch { /* ignore */ }
  }
  renderList();
  renderState();
}

// ---------- logs ----------
async function resetLog(env) {
  log = { id: env.id, file: null, offset: null, lines: [] };
  $('log').textContent = '';
  const files = (await api.listLogs(env.id)) || [];
  $('log-file').replaceChildren(...files.map((f) => new Option(f, f)));
  log.file = files[0] ?? null;
  if (!log.file) $('log').textContent = 'Sin logs todavía (crx-quickstart/logs no existe).';
  await pollLog();
}

function levelAllowed(line, min) {
  if (!min) return true;
  const order = ['DEBUG', 'INFO', 'WARN', 'ERROR'];
  const m = line.match(/\*(DEBUG|INFO|WARN|ERROR)\*/);
  if (!m) return true; // continuation lines (stack traces) stay visible
  return order.indexOf(m[1]) >= order.indexOf(min);
}

function renderLog() {
  const q = $('log-filter').value.toLowerCase();
  const lvl = $('log-level').value;
  const out = log.lines.filter((l) => levelAllowed(l, lvl) && (!q || l.toLowerCase().includes(q)));
  const el = $('log');
  el.textContent = out.join('\n');
  if ($('log-follow').checked) el.scrollTop = el.scrollHeight;
}

async function pollLog() {
  if (!log.id || !log.file) return;
  const { id, file } = log;
  try {
    const chunk = await api.readLog(id, file, log.offset);
    if (log.id !== id || log.file !== file) return; // user switched meanwhile
    const reset = log.offset !== null && chunk.offset < log.offset; // rotated
    if (reset) log.lines = [];
    log.offset = chunk.offset;
    if (chunk.text) {
      const parts = chunk.text.split('\n');
      if (log.partial) { parts[0] = log.partial + parts[0]; }
      log.partial = parts.pop(); // incomplete trailing line
      log.lines.push(...parts);
      while (log.lines.join('\n').length > MAX_LOG_CHARS) log.lines.splice(0, 500);
      renderLog();
    }
  } catch (e) { /* file may not exist yet */ }
}

// ---------- dialog ----------
const OTHER = '__other__';

// Fill the JAR selector from the *.jar files in the chosen folder.
// 0 jars -> ask for one (it will be copied into the folder on save); 1 -> auto-select; >1 -> user chooses.
async function loadJars(dir, current = '') {
  const jars = dir ? ((await api.scanJars(dir)) || []) : [];
  const sel = $('jar-choice');
  const opts = [];
  if (jars.length > 1 && !jars.includes(current)) opts.push(new Option('— Elige el JAR de AEM —', ''));
  opts.push(...jars.map((j) => new Option(j, j)));
  opts.push(new Option('Otro JAR (se copiará a la carpeta)…', OTHER));
  sel.replaceChildren(...opts);
  sel.value = jars.includes(current) ? current : jars.length === 1 ? jars[0] : jars.length === 0 ? OTHER : '';
  $('jar-path').value = '';
  $('jar-hint').textContent = !dir ? '' : jars.length === 0
    ? 'No hay ningún JAR en la carpeta: indica dónde está; se copiará a la carpeta del entorno.'
    : jars.length > 1 ? 'Hay varios JAR en la carpeta: elige cuál es el de AEM.' : '';
  updateJarUI();
}
function updateJarUI() { $('jar-other').hidden = $('jar-choice').value !== OTHER; }
const chosenJar = () => ($('jar-choice').value === OTHER ? $('jar-path').value : $('jar-choice').value);

// ---- multi-step form: one <fieldset.step> visible at a time ----
const stepEls = () => [...document.querySelectorAll('#form fieldset.step')];
let step = 0;
let maxStep = 0;   // furthest step reached; steps up to it can be revisited by clicking the header

function showStep(n) {
  const els = stepEls();
  step = Math.max(0, Math.min(n, els.length - 1));
  maxStep = Math.max(maxStep, step);
  els.forEach((el, i) => el.classList.toggle('active', i === step));
  [...$('steps').children].forEach((li, i) => {
    li.classList.toggle('active', i === step);
    li.classList.toggle('done', i !== step && i <= maxStep);
  });
  const last = step === els.length - 1;
  $('btn-back').hidden = step === 0;
  $('btn-next').textContent = last ? (editing ? 'Guardar' : 'Crear entorno') : 'Siguiente';
  els[step].querySelector('input:not([readonly]), select')?.focus();
}

// Validates only the fields of step `n`; returns an error message or null.
function validateStep(n) {
  const el = stepEls()[n];
  for (const input of el.querySelectorAll('input, select')) {
    if (!input.checkValidity()) { input.reportValidity(); return 'invalid'; }
  }
  if (n === 1) {
    if ($('jar-choice').value === '') return 'Elige el JAR de AEM.';
    if ($('jar-choice').value === OTHER && !$('jar-path').value) return 'Selecciona el JAR de AEM (o deja la carpeta con uno dentro).';
  }
  return null;
}

async function goNext() {
  const err = validateStep(step);
  if (err) { if (err !== 'invalid') await api.alert(err); return false; }
  if (step < stepEls().length - 1) { showStep(step + 1); return false; }
  return true; // last step validated => caller saves
}

$('btn-back').onclick = () => showStep(step - 1);
$('steps').onclick = (ev) => {
  const i = [...$('steps').children].indexOf(ev.target);
  if (i >= 0 && i <= maxStep && (i < step || !validateStep(step))) showStep(i);
};

function openDialog(env) {
  editing = env;
  const f = $('form');
  $('dlg-title').textContent = env ? 'Editar entorno' : 'Nuevo entorno';
  $('btn-delete').hidden = !env;
  $('env-ws').replaceChildren(...workspaces.map((w) => new Option(w.name, w.id)));
  const d = env || { workspaceId: currentWs, name: '', kind: 'author', dir: '', jar: '', port: 4502, extraRunmodes: '', jvmOpts: '', startCommand: '', stopCommand: '' };
  for (const k of Object.keys(d)) if (k !== 'jar' && f.elements[k]) f.elements[k].value = d[k];
  loadJars(d.dir, d.jar);
  maxStep = env ? stepEls().length - 1 : 0; // editing: every step is reachable
  showStep(0);
  $('dlg').showModal();
}

$('form').elements.kind.onchange = (ev) => {
  if (editing) return;
  $('form').elements.port.value = ev.target.value === 'author' ? 4502 : 4503;
};
$('pick-dir').onclick = guard(async () => {
  const dir = await api.pickDirectory();
  if (dir) { $('form').elements.dir.value = dir; await loadJars(dir); }
});
$('form').elements.dir.onchange = (ev) => loadJars(ev.target.value.trim());
$('jar-choice').onchange = updateJarUI;
$('pick-jar').onclick = guard(async () => {
  const jar = await api.pickJar();
  if (jar) $('jar-path').value = jar;
});
$('btn-cancel').onclick = () => $('dlg').close();
$('btn-add').onclick = () => openDialog(null);
$('btn-edit').onclick = () => openDialog(selected());
$('btn-delete').onclick = guard(async () => {
  if (!(await api.confirm(`¿Eliminar "${editing.name}" de la lista? (No borra archivos)`))) return;
  await api.deleteEnvironment(editing.id);
  $('dlg').close();
  await refreshEnvs();
});
$('form').onsubmit = async (ev) => {
  ev.preventDefault(); // keep the dialog open if validation or saving fails
  try {
  if (!(await goNext())) return; // not on the last step yet (Enter/Siguiente just advances)
  for (let i = 0; i < step; i++) { const e = validateStep(i); if (e) { showStep(i); if (e !== 'invalid') await api.alert(e); return; } }
  const f = $('form').elements;
  const env = {
    id: editing?.id || crypto.randomUUID(), workspaceId: f.workspaceId.value,
    name: f.name.value.trim(), kind: f.kind.value, dir: f.dir.value.trim(), jar: chosenJar(), port: Number(f.port.value),
    extraRunmodes: f.extraRunmodes.value.trim(), jvmOpts: f.jvmOpts.value.trim(),
    startCommand: f.startCommand.value.trim(), stopCommand: f.stopCommand.value.trim(),
  };
  await api.saveEnvironment(env);
  selectedId = env.id;
  currentWs = env.workspaceId;
  safeStore.set('workspace', currentWs);
  await refreshEnvs();
  await pollStatuses();
  $('dlg').close();
  } catch (e) { await api.alert(String(e)); }
};

// ---------- workspaces ----------
let editingWs = null;    // workspace being edited (null = new)

function openWsDialog(ws) {
  editingWs = ws;
  $('ws-title').textContent = ws ? 'Renombrar workspace' : 'Nuevo workspace';
  $('ws-name').value = ws ? ws.name : '';
  $('ws-delete').hidden = !ws;
  $('ws-dlg').showModal();
}
$('ws-select').onchange = (ev) => {
  currentWs = ev.target.value;
  safeStore.set('workspace', currentWs);
  selectedId = visibleEnvs()[0]?.id ?? null;
  log.id = null;
  renderList();
  renderDetail();
};
$('ws-add').onclick = () => openWsDialog(null);
$('ws-edit').onclick = () => openWsDialog(workspaces.find((w) => w.id === currentWs));
$('ws-cancel').onclick = () => $('ws-dlg').close();
$('ws-delete').onclick = async () => {
  try {
    const n = envs.filter((e) => e.workspaceId === editingWs.id).length;
    if (!(await api.confirm(`¿Eliminar el workspace "${editingWs.name}" y sus ${n} entorno(s) de la lista? (No borra archivos ni detiene instancias)`))) return;
    await api.deleteWorkspace(editingWs.id);
    $('ws-dlg').close();
    await refreshEnvs();
  } catch (e) { await api.alert(String(e)); }
};
$('ws-form').onsubmit = async (ev) => {
  ev.preventDefault();
  try {
    const ws = { id: editingWs?.id || crypto.randomUUID(), name: $('ws-name').value.trim() };
    if (!ws.name) return;
    await api.saveWorkspace(ws);
    currentWs = ws.id;
    safeStore.set('workspace', currentWs);
    $('ws-dlg').close();
    await refreshEnvs();
  } catch (e) { await api.alert(String(e)); }
};

// ---------- actions ----------
$('btn-start').onclick = guard(async () => {
  await api.start(selectedId);
  statuses[selectedId] = 'starting';
  renderList(); renderState();
});
$('btn-stop').onclick = guard(async () => {
  await api.stop(selectedId);
  statuses[selectedId] = 'stopping';
  renderList(); renderState();
});
$('btn-open').onclick = guard(async () => {
  const e = selected();
  await api.openUrl(`http://localhost:${e.port}/${e.kind === 'author' ? 'aem/start.html' : ''}`);
});
$('btn-folder').onclick = guard(() => api.revealPath(selected().dir));
$('log-file').onchange = (ev) => { log.file = ev.target.value; log.offset = null; log.lines = []; log.partial = ''; pollLog(); };
$('log-filter').oninput = renderLog;
$('log-level').onchange = renderLog;
$('log-clear').onclick = () => { log.lines = []; renderLog(); };

// ---------- boot ----------
(async function boot() {
  await refreshEnvs();
  await pollStatuses();
  setInterval(pollStatuses, 2000);
  setInterval(pollLog, 1000);
  // Log file list changes when the instance first starts: refresh it when state turns running/starting.
  let prev = {};
  setInterval(() => {
    const e = selected();
    if (e && prev[e.id] !== statuses[e.id] && statuses[e.id] !== 'stopped') { prev[e.id] = statuses[e.id]; log.id = null; renderDetail(); }
    else if (e) prev[e.id] = statuses[e.id];
  }, 2500);
})();
