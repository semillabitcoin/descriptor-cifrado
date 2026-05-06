---
phase: 02-spa-frontend-history
plan: 03
type: execute
wave: 2
depends_on: [02-01]
files_modified:
  - frontend/src/App.svelte
  - frontend/src/stores/app.svelte.js
  - frontend/src/lib/api.js
  - frontend/src/lib/clipboard.js
  - frontend/src/components/Header.svelte
  - frontend/src/components/TabBar.svelte
  - frontend/src/components/ThreatModel.svelte
  - frontend/src/components/Modal.svelte
  - frontend/src/components/Toast.svelte
  - frontend/src/components/InlineError.svelte
  - frontend/src/components/Spinner.svelte
  - frontend/src/components/HistoryBadge.svelte
  - frontend/src/components/ThemeToggle.svelte
  - frontend/src/components/HistoryToggle.svelte
autonomous: true
requirements: [UI-02, UI-03, HIST-01]
must_haves:
  truths:
    - "App.svelte renderiza header global + threat model collapsible + tab bar + tabpanel activo"
    - "El theme toggle alterna light/dark/auto y persiste en localStorage 'bed.theme'"
    - "El history toggle alterna ON/OFF y persiste en localStorage 'bed.historyEnabled' default false"
    - "Cuando history toggle ON, aparece badge 'Modo histórico activo' y la tab Historial se renderiza en DOM"
    - "Cuando history toggle OFF, la tab Historial NO se renderiza en DOM (no solo hidden)"
    - "Threat model panel está colapsado por defecto (<details> sin open) y muestra los 3 bloques al expandir"
    - "Tabs implementan ARIA: role=tablist/tab/tabpanel, aria-selected, aria-controls"
    - "Componentes Modal, Toast, InlineError, Spinner están disponibles para los planes 04/05/06"
  artifacts:
    - path: "frontend/src/App.svelte"
      provides: "Root SPA: header + ThreatModel + TabBar + tabpanel switch"
      contains: "TabBar"
    - path: "frontend/src/stores/app.svelte.js"
      provides: "Estado global $state: activeTab, theme, historyEnabled + initFromStorage, setTheme, setHistoryEnabled"
      exports: ["appState", "initFromStorage", "setTheme", "setHistoryEnabled", "setActiveTab"]
    - path: "frontend/src/lib/api.js"
      provides: "Helpers fetch JSON+multipart con manejo unificado de error envelope {error: {code, message}}"
      exports: ["postJson", "postMultipart", "getJson", "deleteJson"]
    - path: "frontend/src/lib/clipboard.js"
      provides: "copyToClipboard con fallback execCommand para contextos no-secure"
      exports: ["copyToClipboard"]
    - path: "frontend/src/components/ThreatModel.svelte"
      provides: "Panel colapsable <details> con callout warning + bloques protege/no protege"
    - path: "frontend/src/components/Modal.svelte"
      provides: "Modal genérico con focus trap, role=dialog, aria-modal, escape closes, default focus en Cancel"
    - path: "frontend/src/components/Toast.svelte"
      provides: "Toast top-right auto-dismiss 3s con aria-live=polite"
    - path: "frontend/src/components/InlineError.svelte"
      provides: "Alert inline arriba del form con role=alert + cerrable"
  key_links:
    - from: "frontend/src/App.svelte"
      to: "frontend/src/stores/app.svelte.js"
      via: "import { appState, initFromStorage } from './stores/app.svelte.js'"
      pattern: "stores/app\\.svelte\\.js"
    - from: "frontend/src/components/Header.svelte"
      to: "frontend/src/stores/app.svelte.js"
      via: "consume appState.theme y appState.historyEnabled"
      pattern: "appState"
    - from: "frontend/src/components/TabBar.svelte"
      to: "frontend/src/stores/app.svelte.js"
      via: "lee appState.activeTab y appState.historyEnabled"
      pattern: "activeTab"
---

<objective>
Construir el shell de la SPA: App.svelte (root), Header con toggles, TabBar con ARIA, ThreatModel collapsible, y los componentes compartidos (Modal, Toast, InlineError, Spinner) que las planes 04/05/06 consumirán para Cifrar, Descifrar e Historial. Establece el store global Svelte 5 (`stores/app.svelte.js`) con persistencia localStorage y los helpers `lib/api.js` (fetch wrappers) + `lib/clipboard.js` (copy con fallback).

Purpose: Cubrir UI-02 (estructura de tabs), UI-03 (threat model visible) y HIST-01 (toggle modo histórico con default OFF). También establece los contratos de componentes que los siguientes planes consumirán sin reinventar.
Output: Shell funcional con tabs vacías (los TabPanels los crean planes 04/05/06), threat model y toggles operativos, componentes compartidos listos.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/02-spa-frontend-history/02-CONTEXT.md
@.planning/phases/02-spa-frontend-history/02-RESEARCH.md
@.planning/phases/02-spa-frontend-history/02-UI-SPEC.md
@frontend/src/App.svelte
@frontend/src/lib/tokens.css
@frontend/src/app.css
@frontend/index.html
</context>

<tasks>

<task type="auto" tdd="false">
  <name>Task 1: Store global Svelte 5 + helpers lib/api + lib/clipboard</name>
  <files>frontend/src/stores/app.svelte.js, frontend/src/lib/api.js, frontend/src/lib/clipboard.js</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 3 — Svelte 5 Runes; Patrón 7 — Clipboard API fallback)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-03 theme persist, D-18 historyEnabled persist, D-06 state-based routing, D-16/D-17 nada se persiste excepto theme y historyEnabled)
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§LocalStorage Keys — solo bed.theme y bed.historyEnabled, ninguna otra)
    - frontend/src/lib/tokens.css (custom property data-theme="dark" — confirmar el atributo a setear)
  </read_first>
  <action>
1. **`frontend/src/stores/app.svelte.js`** — store global. Svelte 5 usa archivos `.svelte.js` (extensión especial) para que el compilador procese runes `$state`. Contenido EXACTO:

```javascript
// Estado global de la SPA (Svelte 5 runes pattern).
// Importar desde componentes: import { appState, ... } from '../stores/app.svelte.js';
//
// localStorage keys (UI-SPEC §LocalStorage Keys):
//   bed.theme           ∈ "light" | "dark" | "auto"   default "auto"
//   bed.historyEnabled  ∈ "true"  | "false"            default "false"
// NINGUNA otra clave se persiste — descriptor, xpub, resultados viven solo en memoria (D-16, D-17).

const VALID_TABS = ['cifrar', 'descifrar', 'historial'];
const VALID_THEMES = ['light', 'dark', 'auto'];

export const appState = $state({
  activeTab: 'cifrar',
  theme: 'auto',
  historyEnabled: false,
});

function applyThemeToDom(theme) {
  // tokens.css usa :root[data-theme="dark"] y :root:not([data-theme="light"]) @media dark.
  // Para 'auto' eliminamos el atributo (los media queries de tokens.css cubren auto).
  // Para 'light' y 'dark' lo seteamos explícitamente.
  const root = document.documentElement;
  if (theme === 'auto') {
    root.removeAttribute('data-theme');
  } else {
    root.setAttribute('data-theme', theme);
  }
}

export function initFromStorage() {
  try {
    const t = localStorage.getItem('bed.theme');
    if (VALID_THEMES.includes(t)) {
      appState.theme = t;
    }
    const h = localStorage.getItem('bed.historyEnabled');
    appState.historyEnabled = h === 'true';
  } catch {
    // localStorage no disponible (private mode, file://, etc.) — usar defaults.
  }
  applyThemeToDom(appState.theme);
}

export function setTheme(theme) {
  if (!VALID_THEMES.includes(theme)) return;
  appState.theme = theme;
  try {
    localStorage.setItem('bed.theme', theme);
  } catch {}
  applyThemeToDom(theme);
}

export function setHistoryEnabled(enabled) {
  appState.historyEnabled = !!enabled;
  try {
    localStorage.setItem('bed.historyEnabled', String(!!enabled));
  } catch {}
  // Si el usuario apaga el toggle estando en la tab Historial, volver a Cifrar (D-20).
  if (!enabled && appState.activeTab === 'historial') {
    appState.activeTab = 'cifrar';
  }
}

export function setActiveTab(tab) {
  if (!VALID_TABS.includes(tab)) return;
  // No permitir activar 'historial' si el toggle está OFF.
  if (tab === 'historial' && !appState.historyEnabled) return;
  appState.activeTab = tab;
}
```

2. **`frontend/src/lib/api.js`** — wrappers fetch que extraen el envelope `{error: {code, message}}` del backend (Phase 1 D-16/D-17) y normalizan errores de red. Contenido EXACTO:

```javascript
// Wrappers fetch para los endpoints axum.
// Backend retorna body de error con shape: { "error": { "code": "<UPPER_SNAKE>", "message": "<castellano>" } }
// Estos wrappers lanzan ApiError con .code, .message, .status — los componentes los muestran tal cual.

export class ApiError extends Error {
  constructor({ status, code, message }) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
  }
}

async function unwrap(resp) {
  const ct = resp.headers.get('content-type') || '';
  if (resp.ok) {
    if (resp.status === 204) return null;
    if (ct.includes('application/json')) return resp.json();
    return resp.text();
  }
  // Error path
  let code = 'HTTP_ERROR';
  let message = `Error ${resp.status}`;
  if (ct.includes('application/json')) {
    try {
      const body = await resp.json();
      code = body?.error?.code ?? code;
      message = body?.error?.message ?? message;
    } catch {}
  }
  throw new ApiError({ status: resp.status, code, message });
}

function networkError(e) {
  // fetch lanza TypeError cuando hay error de red, abort, CORS, etc.
  const message = 'No se pudo conectar al servidor local. Comprueba que la app esté en ejecución.';
  return new ApiError({ status: 0, code: 'NETWORK_ERROR', message });
}

export async function postJson(url, body) {
  let resp;
  try {
    resp = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
  } catch (e) {
    throw networkError(e);
  }
  return unwrap(resp);
}

export async function postMultipart(url, formData) {
  let resp;
  try {
    resp = await fetch(url, { method: 'POST', body: formData });
  } catch (e) {
    throw networkError(e);
  }
  return unwrap(resp);
}

export async function getJson(url) {
  let resp;
  try {
    resp = await fetch(url, { method: 'GET' });
  } catch (e) {
    throw networkError(e);
  }
  return unwrap(resp);
}

export async function deleteJson(url) {
  let resp;
  try {
    resp = await fetch(url, { method: 'DELETE' });
  } catch (e) {
    throw networkError(e);
  }
  return unwrap(resp);
}
```

3. **`frontend/src/lib/clipboard.js`** — copy con fallback `execCommand` (Patrón 7 RESEARCH — Tor Browser Chromium HTTP no tiene `navigator.clipboard`):

```javascript
// Copia texto al portapapeles con fallback para contextos no-secure (Tor onion en Chromium, LAN HTTP).
// Retorna true si la copia tuvo éxito, false si falló completamente.

export async function copyToClipboard(text) {
  if (navigator.clipboard && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // Permission denied or DOMException — fall through to execCommand.
    }
  }
  // Fallback: execCommand('copy') sobre un textarea efímero
  const textarea = document.createElement('textarea');
  textarea.value = text;
  textarea.setAttribute('readonly', '');
  textarea.style.position = 'fixed';
  textarea.style.top = '0';
  textarea.style.left = '0';
  textarea.style.opacity = '0';
  textarea.style.pointerEvents = 'none';
  document.body.appendChild(textarea);
  textarea.focus();
  textarea.select();
  let ok = false;
  try {
    ok = document.execCommand('copy');
  } catch {
    ok = false;
  }
  document.body.removeChild(textarea);
  return ok;
}
```

NO usar Svelte 4 `writable()` stores — usa `$state` runes (Trampa anti-patrón en RESEARCH §Patrón 3).
NO leer/escribir localStorage para nada que no sea `bed.theme` y `bed.historyEnabled` (D-16, D-17).
NO añadir tracking ni telemetría en api.js (PROYECTO no tiene telemetry).
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; node -e "const f = require('fs'); const s = f.readFileSync('src/stores/app.svelte.js','utf8'); if (!s.includes('\$state')) process.exit(1); if (!s.includes('bed.theme')) process.exit(1); if (!s.includes('bed.historyEnabled')) process.exit(1); console.log('store OK');" &amp;&amp; node -e "const f = require('fs'); const s = f.readFileSync('src/lib/api.js','utf8'); if (!s.includes('ApiError')) process.exit(1); if (!s.includes('postJson')) process.exit(1); console.log('api OK');"</automated>
  </verify>
  <acceptance_criteria>
    - `grep "export const appState = \\\$state" /workspace/descriptor-cifrado/frontend/src/stores/app.svelte.js` encuentra match (Svelte 5 rune)
    - `grep "bed.theme" /workspace/descriptor-cifrado/frontend/src/stores/app.svelte.js` encuentra match
    - `grep "bed.historyEnabled" /workspace/descriptor-cifrado/frontend/src/stores/app.svelte.js` encuentra match
    - `grep "historyEnabled: false" /workspace/descriptor-cifrado/frontend/src/stores/app.svelte.js` encuentra match (default OFF, HIST-01)
    - `grep "data-theme" /workspace/descriptor-cifrado/frontend/src/stores/app.svelte.js` encuentra match (DOM apply)
    - `! grep "writable" /workspace/descriptor-cifrado/frontend/src/stores/app.svelte.js` (no Svelte 4 anti-pattern)
    - `grep "export class ApiError" /workspace/descriptor-cifrado/frontend/src/lib/api.js` encuentra match
    - `grep "export async function postJson" /workspace/descriptor-cifrado/frontend/src/lib/api.js` encuentra match
    - `grep "export async function postMultipart" /workspace/descriptor-cifrado/frontend/src/lib/api.js` encuentra match
    - `grep "export async function getJson" /workspace/descriptor-cifrado/frontend/src/lib/api.js` encuentra match
    - `grep "export async function deleteJson" /workspace/descriptor-cifrado/frontend/src/lib/api.js` encuentra match
    - `grep "navigator.clipboard" /workspace/descriptor-cifrado/frontend/src/lib/clipboard.js` encuentra match
    - `grep "execCommand" /workspace/descriptor-cifrado/frontend/src/lib/clipboard.js` encuentra match (fallback)
    - `grep "isSecureContext" /workspace/descriptor-cifrado/frontend/src/lib/clipboard.js` encuentra match
  </acceptance_criteria>
  <done>Store global Svelte 5 con $state + persistencia localStorage para theme y historyEnabled solamente; api.js wrappers fetch con extracción de error envelope; clipboard.js con fallback execCommand para contextos no-secure.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 2: Componentes compartidos (Modal, Toast, InlineError, Spinner, ThreatModel, ThemeToggle, HistoryToggle, HistoryBadge)</name>
  <files>frontend/src/components/Modal.svelte, frontend/src/components/Toast.svelte, frontend/src/components/InlineError.svelte, frontend/src/components/Spinner.svelte, frontend/src/components/ThreatModel.svelte, frontend/src/components/ThemeToggle.svelte, frontend/src/components/HistoryToggle.svelte, frontend/src/components/HistoryBadge.svelte</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (TODAS las secciones — §Component Contracts: Buttons, Toggle, Threat Model Panel, Modal, Toast, Inline Error Alert, Spinner, Border Radius, Shadows, Transitions; §Copywriting Contract; §Accessibility Requirements)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-30, D-31, D-32 threat model; D-33 spinner inline; D-34 toast; D-35 inline error; D-36 modal con focus en Cancel; D-37 a11y)
    - frontend/src/lib/tokens.css (custom properties para usar)
    - frontend/src/stores/app.svelte.js (consumido por ThemeToggle y HistoryToggle)
  </read_first>
  <action>
Crea los 8 componentes compartidos. Cada uno usa SOLO custom properties de `tokens.css` (no hex inventados) y el copy LITERAL del UI-SPEC §Copywriting Contract. Componentes son hand-built (sin librería externa, sin shadcn — D-01 UI-SPEC).

1. **`frontend/src/components/Spinner.svelte`** — spinner inline 16px del UI-SPEC §Spinner:

```svelte
<script>
  let { size = 16, color = 'var(--color-accent-fg)' } = $props();
</script>

<span
  class="spinner"
  role="status"
  aria-label="Cargando"
  style:--spinner-size="{size}px"
  style:--spinner-color={color}
></span>

<style>
  .spinner {
    display: inline-block;
    width: var(--spinner-size);
    height: var(--spinner-size);
    border: 2px solid transparent;
    border-top-color: var(--spinner-color);
    border-right-color: var(--spinner-color);
    border-radius: 50%;
    animation: spin 600ms linear infinite;
    vertical-align: -3px;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  @media (prefers-reduced-motion: reduce) {
    .spinner { animation-duration: 2400ms; }
  }
</style>
```

2. **`frontend/src/components/Toast.svelte`** — toast top-right 320px con auto-dismiss 3s (UI-SPEC §Toast):

```svelte
<script>
  let { message = '', visible = $bindable(false), durationMs = 3000 } = $props();

  let timer;
  $effect(() => {
    if (visible) {
      clearTimeout(timer);
      timer = setTimeout(() => { visible = false; }, durationMs);
    }
    return () => clearTimeout(timer);
  });
</script>

{#if visible}
  <div class="toast" role="status" aria-live="polite">{message}</div>
{/if}

<style>
  .toast {
    position: fixed;
    top: var(--space-md);
    right: var(--space-md);
    width: 320px;
    padding: var(--space-sm-plus) var(--space-md);
    border-radius: var(--radius-toast);
    background: var(--color-toast-bg);
    color: var(--color-toast-text);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    line-height: var(--line-height-label);
    box-shadow: var(--shadow-modal);
    z-index: 9999;
    animation: slide-in var(--transition-toast-in);
  }
  @keyframes slide-in {
    from { transform: translateX(120%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
  @media (prefers-reduced-motion: reduce) {
    .toast { animation: none; }
  }
</style>
```

3. **`frontend/src/components/InlineError.svelte`** — alerta inline arriba de form (UI-SPEC §Inline Error Alert):

```svelte
<script>
  let { message = '', visible = $bindable(false) } = $props();
  function close() { visible = false; }
</script>

{#if visible && message}
  <div class="alert" role="alert">
    <span class="icon" aria-hidden="true">⚠</span>
    <span class="message">{message}</span>
    <button type="button" class="close" aria-label="Cerrar mensaje de error" onclick={close}>×</button>
  </div>
{/if}

<style>
  .alert {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    padding: var(--space-sm-plus) var(--space-md);
    background: var(--color-warning-bg);
    border-left: 4px solid var(--color-warning-border);
    border-radius: var(--radius-card);
    color: var(--color-warning-text);
    font-size: var(--font-size-body);
    line-height: var(--line-height-body);
  }
  .icon { font-size: 18px; line-height: 1; }
  .message { flex: 1; }
  .close {
    background: transparent;
    border: 0;
    color: inherit;
    font-size: 20px;
    cursor: pointer;
    padding: 0 var(--space-xs);
    min-width: var(--touch-target);
    min-height: var(--touch-target);
  }
  .close:hover { background: rgba(0,0,0,0.06); border-radius: var(--radius-button); }
</style>
```

4. **`frontend/src/components/Modal.svelte`** — modal genérico con focus trap, escape close, default focus en Cancel (UI-SPEC §Modal, D-36):

```svelte
<script>
  let {
    open = $bindable(false),
    title = '',
    children,
    onConfirm,
    onCancel,
    confirmLabel = 'Aceptar',
    cancelLabel = 'Cancelar',
    confirmVariant = 'primary',  // 'primary' | 'destructive'
    confirmLoading = false,
  } = $props();

  let panel;
  let cancelBtn;

  $effect(() => {
    if (open) {
      // Focus default → cancelBtn (D-36)
      queueMicrotask(() => cancelBtn?.focus());
      function onKey(e) {
        if (e.key === 'Escape') { e.preventDefault(); cancel(); }
        if (e.key === 'Tab') trapFocus(e);
      }
      document.addEventListener('keydown', onKey);
      return () => document.removeEventListener('keydown', onKey);
    }
  });

  function trapFocus(e) {
    if (!panel) return;
    const focusables = panel.querySelectorAll('button, [href], input, [tabindex]:not([tabindex="-1"])');
    if (!focusables.length) return;
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault(); last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault(); first.focus();
    }
  }

  function cancel() {
    open = false;
    onCancel?.();
  }
  function confirm() {
    onConfirm?.();
  }
</script>

{#if open}
  <div class="backdrop" onclick={cancel} role="presentation"></div>
  <div
    class="panel"
    bind:this={panel}
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
  >
    <h2 id="modal-title" class="title">{title}</h2>
    <div class="body">
      {@render children?.()}
    </div>
    <div class="actions">
      <button
        type="button"
        class="btn btn-secondary"
        bind:this={cancelBtn}
        onclick={cancel}
        disabled={confirmLoading}
      >{cancelLabel}</button>
      <button
        type="button"
        class="btn btn-{confirmVariant}"
        onclick={confirm}
        disabled={confirmLoading}
      >{confirmLabel}</button>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.4);
    backdrop-filter: blur(2px);
    z-index: 9000;
  }
  .panel {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    background: var(--color-surface-raised);
    color: var(--color-text-primary);
    border-radius: var(--radius-modal);
    padding: var(--space-lg);
    max-width: 400px;
    width: calc(100% - var(--space-xl));
    box-shadow: var(--shadow-modal);
    z-index: 9001;
  }
  .title {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-heading);
  }
  .body {
    font-size: var(--font-size-body);
    line-height: var(--line-height-body);
    margin-bottom: var(--space-lg);
  }
  .actions {
    display: flex; gap: var(--space-sm); justify-content: flex-end;
  }
  .btn {
    min-height: var(--touch-target);
    min-width: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    cursor: pointer;
    transition: background-color var(--transition-color), border-color var(--transition-color), color var(--transition-color);
  }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-secondary {
    background: transparent;
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }
  .btn-secondary:hover:not(:disabled) { background: var(--color-surface-sunken); }
  .btn-primary {
    background: var(--color-accent);
    color: var(--color-accent-fg);
    border: 0;
  }
  .btn-primary:hover:not(:disabled) { background: var(--color-accent-hover); }
  .btn-destructive {
    background: var(--color-destructive);
    color: var(--color-destructive-fg);
    border: 0;
  }
  .btn-destructive:hover:not(:disabled) { background: var(--color-destructive-hover); }
</style>
```

5. **`frontend/src/components/ThreatModel.svelte`** — `<details>` colapsable con copy LITERAL del UI-SPEC §Threat Model Copy (D-31):

```svelte
<details class="threat">
  <summary>
    <span class="icon" aria-hidden="true">⚠</span>
    <span>Modelo de amenazas</span>
  </summary>
  <div class="content">
    <div class="callout">
      <strong>Ninguna ubicación debe contener simultáneamente el .bed y una xpub del multisig.</strong>
    </div>
    <div class="protege">
      <span class="prefix">✓ Protege:</span>
      Tu descriptor multisig contra un atacante que solo encuentre el archivo .bed.
    </div>
    <div class="caution">
      <span class="prefix">✗ No protege:</span>
      <ul>
        <li>Un atacante que comprometa StartOS durante el cifrado (el descriptor pasa por memoria del proceso).</li>
        <li>Un atacante que ya tenga una xpub de tu multisig.</li>
      </ul>
    </div>
    <a class="readme-link" href="#readme-threat-model">Ver modelo de amenazas completo →</a>
  </div>
</details>

<style>
  .threat {
    background: var(--color-surface-raised);
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-md) var(--space-lg);
  }
  summary {
    list-style: none;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    min-height: var(--touch-target);
    transition: background-color var(--transition-color);
  }
  summary::-webkit-details-marker { display: none; }
  summary:hover { background: var(--color-surface-sunken); color: var(--color-text-primary); }
  .icon { font-size: 16px; }
  .content {
    margin-top: var(--space-md);
    display: flex; flex-direction: column; gap: var(--space-md);
    font-size: var(--font-size-body);
    line-height: var(--line-height-body);
  }
  .callout {
    background: var(--color-warning-bg);
    border-left: 4px solid var(--color-warning-border);
    color: var(--color-warning-text);
    padding: var(--space-md);
    border-radius: var(--radius-card);
  }
  .protege {
    background: var(--color-success-bg);
    color: var(--color-success-text);
    padding: var(--space-md);
    border-radius: var(--radius-card);
  }
  .caution {
    background: var(--color-caution-bg);
    color: var(--color-caution-text);
    padding: var(--space-md);
    border-radius: var(--radius-card);
  }
  .caution ul { margin: var(--space-sm) 0 0 0; padding-left: var(--space-lg); }
  .prefix { font-weight: var(--font-weight-bold); margin-right: var(--space-xs); }
  .readme-link {
    color: var(--color-accent);
    text-decoration: underline;
    font-size: var(--font-size-label);
  }
  .readme-link:hover { color: var(--color-accent-hover); }
</style>
```

6. **`frontend/src/components/ThemeToggle.svelte`** — toggle 3-estados light/dark/auto (D-03):

```svelte
<script>
  import { appState, setTheme } from '../stores/app.svelte.js';

  function cycle() {
    const next = appState.theme === 'auto' ? 'light' : appState.theme === 'light' ? 'dark' : 'auto';
    setTheme(next);
  }

  const labels = { light: 'Tema claro', dark: 'Tema oscuro', auto: 'Tema automático' };
  const icons = { light: '☀', dark: '☾', auto: '◐' };
</script>

<button
  type="button"
  class="theme-toggle"
  aria-label={labels[appState.theme]}
  title={labels[appState.theme]}
  onclick={cycle}
>
  <span aria-hidden="true">{icons[appState.theme]}</span>
</button>

<style>
  .theme-toggle {
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text-primary);
    border-radius: var(--radius-button);
    min-width: var(--touch-target);
    min-height: var(--touch-target);
    padding: var(--space-sm);
    cursor: pointer;
    font-size: 18px;
    transition: background-color var(--transition-color), border-color var(--transition-color);
  }
  .theme-toggle:hover { background: var(--color-surface-sunken); }
</style>
```

7. **`frontend/src/components/HistoryToggle.svelte`** — pill toggle 44×24 con role=switch (UI-SPEC §Toggle, D-18):

```svelte
<script>
  import { appState, setHistoryEnabled } from '../stores/app.svelte.js';
</script>

<label class="wrap">
  <span class="text">Historial</span>
  <button
    type="button"
    role="switch"
    class="track"
    class:on={appState.historyEnabled}
    aria-checked={appState.historyEnabled}
    aria-label="Modo histórico"
    onclick={() => setHistoryEnabled(!appState.historyEnabled)}
  >
    <span class="thumb" aria-hidden="true"></span>
  </button>
</label>

<style>
  .wrap {
    display: inline-flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
  }
  .text {
    font-size: var(--font-size-label);
    color: var(--color-text-primary);
    line-height: var(--line-height-label);
  }
  .track {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: var(--radius-pill);
    background: var(--color-border);
    border: 0;
    cursor: pointer;
    padding: 0;
    transition: background-color var(--transition-toggle);
  }
  .track.on { background: var(--color-accent); }
  .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: white;
    transition: left var(--transition-toggle);
    box-shadow: 0 1px 2px rgba(0,0,0,0.2);
  }
  .track.on .thumb { left: 23px; }
</style>
```

8. **`frontend/src/components/HistoryBadge.svelte`** — badge "Modo histórico activo" cuando ON (D-18):

```svelte
<script>
  import { appState } from '../stores/app.svelte.js';
</script>

{#if appState.historyEnabled}
  <span class="badge" aria-label="Modo histórico activo">Modo histórico activo</span>
{/if}

<style>
  .badge {
    display: inline-block;
    padding: var(--space-xs) var(--space-sm);
    background: var(--color-history-badge-bg);
    color: var(--color-history-badge-text);
    border-radius: var(--radius-pill);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    line-height: 1;
  }
</style>
```

NO inventar emojis adicionales — usa los del UI-SPEC (✓ ✗ ⚠).
NO uses íconos externos (Heroicons, Lucide, etc.) — solo SVG inline o caracteres unicode.
TODO el copy LITERAL del UI-SPEC §Copywriting Contract — castellano, no argentino.
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; npm run build 2>&amp;1 | tail -10 &amp;&amp; test -f src/components/Modal.svelte &amp;&amp; test -f src/components/Toast.svelte &amp;&amp; test -f src/components/InlineError.svelte &amp;&amp; test -f src/components/ThreatModel.svelte &amp;&amp; test -f src/components/ThemeToggle.svelte &amp;&amp; test -f src/components/HistoryToggle.svelte &amp;&amp; test -f src/components/HistoryBadge.svelte &amp;&amp; test -f src/components/Spinner.svelte</automated>
  </verify>
  <acceptance_criteria>
    - `grep "role=\"dialog\"" /workspace/descriptor-cifrado/frontend/src/components/Modal.svelte` encuentra match
    - `grep "aria-modal=\"true\"" /workspace/descriptor-cifrado/frontend/src/components/Modal.svelte` encuentra match
    - `grep 'role="alert"' /workspace/descriptor-cifrado/frontend/src/components/InlineError.svelte` encuentra match
    - `grep 'aria-live="polite"' /workspace/descriptor-cifrado/frontend/src/components/Toast.svelte` encuentra match
    - `grep "Ninguna ubicación debe contener simultáneamente el .bed y una xpub del multisig" /workspace/descriptor-cifrado/frontend/src/components/ThreatModel.svelte` encuentra match (copy literal del UI-SPEC)
    - `grep "✓ Protege:" /workspace/descriptor-cifrado/frontend/src/components/ThreatModel.svelte` encuentra match
    - `grep "✗ No protege:" /workspace/descriptor-cifrado/frontend/src/components/ThreatModel.svelte` encuentra match
    - `grep "<details" /workspace/descriptor-cifrado/frontend/src/components/ThreatModel.svelte` encuentra match (HTML semántico, D-32)
    - `grep "Modo histórico activo" /workspace/descriptor-cifrado/frontend/src/components/HistoryBadge.svelte` encuentra match
    - `grep 'role="switch"' /workspace/descriptor-cifrado/frontend/src/components/HistoryToggle.svelte` encuentra match
    - `grep "aria-checked" /workspace/descriptor-cifrado/frontend/src/components/HistoryToggle.svelte` encuentra match
    - `grep "var(--color-accent)" /workspace/descriptor-cifrado/frontend/src/components/HistoryToggle.svelte` encuentra match (usa tokens, no hex hardcode)
    - `grep "var(--space-md)" /workspace/descriptor-cifrado/frontend/src/components/Toast.svelte` encuentra match
    - `! grep -E "#[0-9A-Fa-f]{6}" /workspace/descriptor-cifrado/frontend/src/components/Modal.svelte` (no hex hardcoded en CSS — todo via tokens)
    - `cd /workspace/descriptor-cifrado/frontend && npm run build` exit code 0
  </acceptance_criteria>
  <done>8 componentes compartidos creados con copy literal del UI-SPEC, custom properties exclusivamente, ARIA correcto y `npm run build` verde.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 3: App.svelte root + Header + TabBar (sin tabpanels — los crean planes 04/05/06)</name>
  <files>frontend/src/App.svelte, frontend/src/components/Header.svelte, frontend/src/components/TabBar.svelte</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§Layout Contract: Page Structure, Tab Panel Widths, §Component Contracts §Tabs, §Header items order)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-05 estructura, D-07 ARIA tabs, D-20 historial conditional)
    - frontend/src/stores/app.svelte.js (estado consumido)
    - frontend/src/components/* (componentes Task 2 — importarlos)
    - frontend/src/lib/tokens.css (variables a usar)
    - frontend/src/App.svelte (placeholder a reemplazar)
  </read_first>
  <action>
1. **`frontend/src/components/Header.svelte`** — header global 56px de altura con logo + threat-model trigger + history toggle + theme toggle (UI-SPEC §Layout — Header items order: logo → spacer → history-badge → history toggle → threat-model button → theme toggle). NOTA: el ThreatModel real es un `<details>` que vive abajo del header (D-30 dice "panel banner abajo del header"), pero el TRIGGER puede vivir en el header. Aquí simplificamos: el ThreatModel completo (con su `<summary>` como trigger) se renderiza en App.svelte como banner directly bajo el header. El header solo lleva los toggles y el badge.

```svelte
<script>
  import HistoryBadge from './HistoryBadge.svelte';
  import HistoryToggle from './HistoryToggle.svelte';
  import ThemeToggle from './ThemeToggle.svelte';
</script>

<header class="header">
  <h1 class="logo">BED</h1>
  <div class="spacer"></div>
  <HistoryBadge />
  <HistoryToggle />
  <ThemeToggle />
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    height: 56px;
    padding: 0 var(--space-md);
    background: var(--color-surface-raised);
    border-bottom: 1px solid var(--color-border);
    box-shadow: var(--shadow-header);
    position: sticky;
    top: 0;
    z-index: 100;
  }
  .logo {
    margin: 0;
    font-size: var(--font-size-display);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-display);
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
  }
  .spacer { flex: 1; }
  @media (max-width: 540px) {
    .header { gap: var(--space-sm); padding: 0 var(--space-sm); }
    .logo { font-size: 20px; }
  }
</style>
```

2. **`frontend/src/components/TabBar.svelte`** — tabs ARIA con role=tablist, role=tab, aria-selected, aria-controls (UI-SPEC §Tabs, D-07). La tab Historial solo se renderiza si `appState.historyEnabled` (D-20):

```svelte
<script>
  import { appState, setActiveTab } from '../stores/app.svelte.js';

  const TABS = [
    { id: 'cifrar', label: 'Cifrar' },
    { id: 'descifrar', label: 'Descifrar' },
  ];

  function handleKey(e, idx) {
    const tabs = visibleTabs;
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      setActiveTab(tabs[(idx + 1) % tabs.length].id);
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      setActiveTab(tabs[(idx - 1 + tabs.length) % tabs.length].id);
    } else if (e.key === 'Home') {
      e.preventDefault();
      setActiveTab(tabs[0].id);
    } else if (e.key === 'End') {
      e.preventDefault();
      setActiveTab(tabs[tabs.length - 1].id);
    }
  }

  let visibleTabs = $derived(
    appState.historyEnabled
      ? [...TABS, { id: 'historial', label: 'Historial' }]
      : TABS
  );
</script>

<nav class="tabbar" role="tablist" aria-label="Secciones principales">
  {#each visibleTabs as tab, idx (tab.id)}
    <button
      type="button"
      role="tab"
      id="tab-{tab.id}"
      class="tab"
      class:active={appState.activeTab === tab.id}
      aria-selected={appState.activeTab === tab.id}
      aria-controls="panel-{tab.id}"
      tabindex={appState.activeTab === tab.id ? 0 : -1}
      onclick={() => setActiveTab(tab.id)}
      onkeydown={(e) => handleKey(e, idx)}
    >
      {tab.label}
    </button>
  {/each}
</nav>

<style>
  .tabbar {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--color-border);
    max-width: 640px;
    margin: 0 auto;
    width: 100%;
    padding: 0 var(--space-md);
  }
  .tab {
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    color: var(--color-text-secondary);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    line-height: var(--line-height-label);
    padding: 0 var(--space-md);
    min-height: var(--touch-target);
    cursor: pointer;
    transition: border-color var(--transition-tab), color var(--transition-tab), font-weight var(--transition-tab);
  }
  .tab:hover { color: var(--color-text-primary); }
  .tab.active {
    color: var(--color-text-primary);
    border-bottom-color: var(--color-accent);
    font-weight: var(--font-weight-bold);
  }
</style>
```

3. **`frontend/src/App.svelte`** — REEMPLAZA completamente el placeholder. Estructura del UI-SPEC §Page Structure: header + threat model banner + tab bar + tabpanel switch. Los tabpanels concretos (Cifrar/Descifrar/Historial) los crean planes 04/05/06; aquí dejamos placeholders identificables pero NO funcionales. Los planes downstream importarán y montarán sus componentes.

```svelte
<script>
  import { onMount } from 'svelte';
  import { appState, initFromStorage } from './stores/app.svelte.js';
  import Header from './components/Header.svelte';
  import ThreatModel from './components/ThreatModel.svelte';
  import TabBar from './components/TabBar.svelte';

  onMount(() => {
    initFromStorage();
  });
</script>

<Header />
<ThreatModel />
<main>
  <TabBar />
  <section
    role="tabpanel"
    id="panel-cifrar"
    aria-labelledby="tab-cifrar"
    class="panel"
    hidden={appState.activeTab !== 'cifrar'}
  >
    <!-- Plan 02-04 monta TabCifrar aquí -->
    <p class="placeholder">Tab Cifrar — pendiente de plan 02-04.</p>
  </section>
  <section
    role="tabpanel"
    id="panel-descifrar"
    aria-labelledby="tab-descifrar"
    class="panel"
    hidden={appState.activeTab !== 'descifrar'}
  >
    <!-- Plan 02-05 monta TabDescifrar aquí -->
    <p class="placeholder">Tab Descifrar — pendiente de plan 02-05.</p>
  </section>
  {#if appState.historyEnabled}
    <section
      role="tabpanel"
      id="panel-historial"
      aria-labelledby="tab-historial"
      class="panel"
      hidden={appState.activeTab !== 'historial'}
    >
      <!-- Plan 02-06 monta TabHistorial aquí -->
      <p class="placeholder">Tab Historial — pendiente de plan 02-06.</p>
    </section>
  {/if}
</main>

<style>
  main {
    min-height: calc(100vh - 56px);
    padding-bottom: var(--space-3xl);
  }
  .panel {
    max-width: 640px;
    margin: 0 auto;
    padding: var(--space-2xl) var(--space-md) var(--space-3xl);
  }
  @media (min-width: 1024px) {
    .panel { padding-left: 0; padding-right: 0; }
  }
  .placeholder {
    color: var(--color-text-secondary);
    font-size: var(--font-size-body);
    text-align: center;
    padding: var(--space-2xl) 0;
  }
</style>
```

4. Compila y verifica: `cd frontend && npm run build`. Asserta tamaño bundle JS+CSS gzipped <50 KB (el constraint del UI-SPEC §Build Constraints; las fonts NO cuentan):

```bash
cd /workspace/descriptor-cifrado/frontend && npm run build
# Inspecciona dist/assets/*.css y *.js
ls -la dist/assets/
# Suma tamaños gzipped de JS+CSS (excluye .woff2)
for f in dist/assets/*.{js,css}; do gzip -c "$f" | wc -c; done | awk '{s+=$1} END {print "Total JS+CSS gzipped:", s, "bytes"}'
# Debe ser <51200 (50 KB)
```

NO añadas funcionalidad de tabs Cifrar/Descifrar/Historial aquí — esto SOLO es el shell.
NO importes los componentes TabCifrar/TabDescifrar/TabHistorial — no existen aún.
NO uses URL hash routing (D-06: state-based interno).
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; npm run build 2>&amp;1 | tail -10 &amp;&amp; SIZE=$(for f in dist/assets/*.js dist/assets/*.css; do [ -f "$f" ] &amp;&amp; gzip -c "$f" | wc -c; done | awk '{s+=$1} END {print s}') &amp;&amp; echo "Bundle JS+CSS gzipped: $SIZE bytes" &amp;&amp; [ "$SIZE" -lt 51200 ]</automated>
  </verify>
  <acceptance_criteria>
    - `grep "import Header" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `grep "import ThreatModel" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `grep "import TabBar" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `grep "initFromStorage" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `grep 'role="tabpanel"' /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra al menos 2 matches (Cifrar + Descifrar)
    - `grep "appState.historyEnabled" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match (gating Historial panel)
    - `grep 'role="tablist"' /workspace/descriptor-cifrado/frontend/src/components/TabBar.svelte` encuentra match
    - `grep 'role="tab"' /workspace/descriptor-cifrado/frontend/src/components/TabBar.svelte` encuentra match
    - `grep "aria-selected" /workspace/descriptor-cifrado/frontend/src/components/TabBar.svelte` encuentra match
    - `grep "aria-controls" /workspace/descriptor-cifrado/frontend/src/components/TabBar.svelte` encuentra match
    - `grep "ArrowRight\\|ArrowLeft" /workspace/descriptor-cifrado/frontend/src/components/TabBar.svelte` encuentra match (keyboard nav)
    - `grep "BED" /workspace/descriptor-cifrado/frontend/src/components/Header.svelte` encuentra match (logo)
    - `grep "HistoryToggle" /workspace/descriptor-cifrado/frontend/src/components/Header.svelte` encuentra match
    - `grep "ThemeToggle" /workspace/descriptor-cifrado/frontend/src/components/Header.svelte` encuentra match
    - `grep "max-width: 640px" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match (UI-SPEC tab panel max-width)
    - `cd /workspace/descriptor-cifrado/frontend && npm run build` exit code 0
    - Bundle JS+CSS gzipped en `dist/assets/` < 51200 bytes (50 KB)
  </acceptance_criteria>
  <done>App.svelte renderiza Header + ThreatModel + TabBar + 3 tabpanels (placeholder text); tabs ARIA correctas con keyboard nav arrow/home/end; history toggle visible en header con badge condicional; threat model colapsado por defecto; bundle JS+CSS <50 KB gzipped.</done>
</task>

</tasks>

<verification>
- `npm run build` produce `dist/index.html` + `dist/assets/*.{js,css,woff2}` con bundle JS+CSS gzipped <50 KB
- TabBar muestra 2 tabs por defecto, 3 tabs cuando localStorage `bed.historyEnabled=true`
- Toggle theme: alternar entre 3 estados aplica `data-theme` al `<html>` y persiste en `bed.theme`
- Threat model: `<details>` colapsable con copy literal del UI-SPEC § Threat Model Copy
- Modal genérico tiene focus trap + Escape close + default focus en Cancel
- Toast aria-live="polite", InlineError role="alert"
</verification>

<success_criteria>
- UI-02: Header con tabs Cifrar y Descifrar (Historial condicional) implementadas con ARIA tablist/tab/tabpanel
- UI-03: Threat model `<details>` colapsable con callout warning + protege ✓ + no protege ✗ + link README
- HIST-01: Toggle modo histórico en header con default OFF, persiste en `bed.historyEnabled`, badge "Modo histórico activo" cuando ON, controla visibilidad de tab Historial
- Componentes Modal/Toast/InlineError/Spinner disponibles para planes 04/05/06
- Helpers lib/api.js (postJson, postMultipart, getJson, deleteJson, ApiError) y lib/clipboard.js (copyToClipboard) listos
- Bundle JS+CSS gzipped <50 KB sin las fuentes
</success_criteria>

<output>
After completion, create `.planning/phases/02-spa-frontend-history/02-03-SUMMARY.md` describing:
- Tamaño actual del bundle gzipped (JS + CSS) — número exacto
- Lista exacta de exports de stores/app.svelte.js que los planes 04/05/06 deben importar
- Lista exacta de exports de lib/api.js y lib/clipboard.js
- Estructura de paneles en App.svelte para que planes 04/05/06 sepan dónde montar sus componentes
- Confirmación: build verde, ARIA roles aplicados, copy literal del UI-SPEC
</output>
