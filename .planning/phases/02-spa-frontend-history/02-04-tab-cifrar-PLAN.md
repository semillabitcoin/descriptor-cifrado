---
phase: 02-spa-frontend-history
plan: 04
type: execute
wave: 3
depends_on: [02-03]
files_modified:
  - frontend/src/App.svelte
  - frontend/src/components/TabCifrar.svelte
  - frontend/src/components/CifrarOutputs.svelte
  - frontend/src/lib/download.js
autonomous: true
requirements: [UI-01, UI-02, HIST-02]
must_haves:
  truths:
    - "Usuario pega un descriptor en textarea y clic 'Cifrar' dispara POST /api/encrypt"
    - "Tras éxito se muestran los TRES outputs simultáneamente: descarga .bed, copia armored, descarga PNG QR"
    - "QR PNG se renderiza inline como <img src=data:image/png;base64,...>"
    - "Copia armored cambia el label a 'Copiado ✓' por 1500ms y dispara toast 'Copiado al portapapeles'"
    - "Si toggle modo histórico ON al cifrar, tras éxito se llama POST /api/history con el bed_b64"
    - "Si POST /api/history falla, toast warning 'Cifrado OK, pero no se guardó en historial' (sin invalidar resultado)"
    - "Errores 422 del backend (DescriptorParse, MissingMultipathWildcard, QrTooLarge) se muestran inline arriba del form con el mensaje literal del backend"
    - "Botón 'Cifrar' muestra spinner inline + label 'Cifrando…' durante la operación"
  artifacts:
    - path: "frontend/src/components/TabCifrar.svelte"
      provides: "Tab Cifrar: textarea descriptor + botón Cifrar + InlineError + zona de resultado"
    - path: "frontend/src/components/CifrarOutputs.svelte"
      provides: "Render de los 3 outputs (download .bed, copy armored, download PNG)"
    - path: "frontend/src/lib/download.js"
      provides: "Helpers triggerDownloadBytes(bytes, filename, mime) y triggerDownloadBase64(b64, filename, mime)"
      exports: ["triggerDownloadBytes", "triggerDownloadBase64"]
  key_links:
    - from: "frontend/src/components/TabCifrar.svelte"
      to: "frontend/src/lib/api.js"
      via: "postJson('/api/encrypt', { descriptor })"
      pattern: "/api/encrypt"
    - from: "frontend/src/components/TabCifrar.svelte"
      to: "frontend/src/lib/api.js"
      via: "postJson('/api/history', { bed_b64 }) cuando historyEnabled (D-12)"
      pattern: "/api/history"
    - from: "frontend/src/components/CifrarOutputs.svelte"
      to: "frontend/src/lib/clipboard.js"
      via: "copyToClipboard(armored)"
      pattern: "copyToClipboard"
    - from: "frontend/src/components/CifrarOutputs.svelte"
      to: "frontend/src/lib/download.js"
      via: "triggerDownloadBase64(bed_b64, 'backup.bed', 'application/octet-stream')"
      pattern: "triggerDownload"
    - from: "frontend/src/App.svelte"
      to: "frontend/src/components/TabCifrar.svelte"
      via: "import + montaje en tabpanel id=panel-cifrar"
      pattern: "TabCifrar"
---

<objective>
Construir la tab "Cifrar" completa: textarea descriptor + botón Cifrar + alerta inline de errores + zona de resultado con los tres outputs (descarga .bed, copia armored con feedback dual toast+label, descarga QR PNG). Integrar la persistencia opt-in del historial cuando el toggle está ON (D-12). Reemplazar el placeholder de TabCifrar en App.svelte por el componente real.

Purpose: Completar el flujo principal de la app — la mayoría de holders abrirá la app, pegará un descriptor, y se llevará el .bed cifrado. Cubre UI-02 (tab Cifrar funcional) y conecta con HIST-02 cuando el toggle está activo.
Output: TabCifrar.svelte funcional, CifrarOutputs.svelte con los 3 outputs y feedback de copy, download.js helpers, App.svelte actualizado para montar TabCifrar.
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
@frontend/src/lib/api.js
@frontend/src/lib/clipboard.js
@frontend/src/stores/app.svelte.js
@frontend/src/components/InlineError.svelte
@frontend/src/components/Toast.svelte
@frontend/src/components/Spinner.svelte

<interfaces>
<!-- Exports the executor consumes — confirmed by Plan 02-03 SUMMARY. -->

From frontend/src/lib/api.js:
```javascript
export class ApiError extends Error { status; code; }
export async function postJson(url, body): any
export async function getJson(url): any
export async function deleteJson(url): any
```

From frontend/src/stores/app.svelte.js:
```javascript
export const appState; // { activeTab, theme, historyEnabled }
export function setActiveTab(tab);
```

From frontend/src/lib/clipboard.js:
```javascript
export async function copyToClipboard(text): Promise<boolean>;
```

Backend contract (Phase 1 D-09) — `POST /api/encrypt`:
- Request: `{ "descriptor": "wsh(...)" }`
- Response 200: `{ "bed_b64": "...", "armored": "-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n...\n-----END BITCOIN ENCRYPTED BACKUP-----\n", "qr_png_b64": "..." }`
- Response 422 (errors envelope): `{ "error": { "code": "MISSING_MULTIPATH_WILDCARD"|"DESCRIPTOR_PARSE"|"QR_TOO_LARGE", "message": "<castellano>" } }`

Backend contract (Plan 02-02) — `POST /api/history`:
- Request: `{ "bed_b64": "..." }`
- Response 200: `{ "id": "<8 hex>", "timestamp": "<ISO>", "filename": "<name>" }`
</interfaces>
</context>

<tasks>

<task type="auto" tdd="false">
  <name>Task 1: lib/download.js helpers + CifrarOutputs.svelte (los 3 outputs con feedback)</name>
  <files>frontend/src/lib/download.js, frontend/src/components/CifrarOutputs.svelte</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 5 — entrega triple output, ejemplo Cifrar result blob; §Ejemplos de código verificados — triggerBedDownload)
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§Buttons, §Output Action Labels, §Toast Messages, §`<pre>` Result Blocks, §Result Zone, §Copywriting Contract)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-09, D-10 sin pre-selección, D-34 copy con toast + label change)
    - frontend/src/lib/clipboard.js (signature copyToClipboard — retorna boolean)
    - frontend/src/components/Toast.svelte (cómo se monta y bindea)
  </read_first>
  <action>
1. **`frontend/src/lib/download.js`** — helpers de descarga client-side. Decode base64 a Uint8Array y trigger download via `<a>`+ `URL.createObjectURL`. Idéntico al ejemplo verificado en RESEARCH §Cifrar result.

```javascript
// Helpers de descarga client-side. Los datos viven solo en memoria del navegador
// hasta que el usuario gatille la descarga (D-16: nada se persiste cliente-side).

function base64ToBytes(b64) {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

export function triggerDownloadBytes(bytes, filename, mime = 'application/octet-stream') {
  const blob = new Blob([bytes], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Revocar tras un tick para que el navegador termine la descarga.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function triggerDownloadBase64(b64, filename, mime = 'application/octet-stream') {
  triggerDownloadBytes(base64ToBytes(b64), filename, mime);
}

export function triggerDownloadText(text, filename, mime = 'text/plain') {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
```

2. **`frontend/src/components/CifrarOutputs.svelte`** — recibe `{ bed_b64, armored, qr_png_b64 }` y renderiza los 3 outputs (D-09, D-10). El feedback de copy es dual: toast 3s ("Copiado al portapapeles" — UI-SPEC §Toast Messages) + label change "Copiado ✓" 1500ms (D-34, UI-SPEC §Output Action Labels):

```svelte
<script>
  import { copyToClipboard } from '../lib/clipboard.js';
  import { triggerDownloadBase64 } from '../lib/download.js';
  import Toast from './Toast.svelte';

  let { result } = $props(); // { bed_b64, armored, qr_png_b64 }

  let copyLabel = $state('Copiar al portapapeles');
  let copyResetTimer;
  let toastVisible = $state(false);
  let toastMessage = $state('');

  function showToast(msg) {
    toastMessage = msg;
    toastVisible = true;
  }

  function downloadBed() {
    // Filename con timestamp para evitar colisiones.
    const ts = new Date().toISOString().replace(/[:.]/g, '-');
    triggerDownloadBase64(result.bed_b64, `backup-${ts}.bed`, 'application/octet-stream');
  }

  function downloadQrPng() {
    const ts = new Date().toISOString().replace(/[:.]/g, '-');
    triggerDownloadBase64(result.qr_png_b64, `backup-${ts}.png`, 'image/png');
  }

  async function handleCopyArmored() {
    const ok = await copyToClipboard(result.armored);
    if (ok) {
      copyLabel = 'Copiado ✓';
      showToast('Copiado al portapapeles');
      clearTimeout(copyResetTimer);
      copyResetTimer = setTimeout(() => { copyLabel = 'Copiar al portapapeles'; }, 1500);
    } else {
      showToast('No se pudo copiar al portapapeles');
    }
  }
</script>

<div class="result-zone" aria-label="Resultado del cifrado">
  <h2 class="title">Resultado</h2>

  <!-- 1. Archivo .bed -->
  <div class="output">
    <div class="row">
      <span class="label">Archivo .bed</span>
      <button type="button" class="btn btn-primary" onclick={downloadBed}>Descargar .bed</button>
    </div>
    <p class="hint">Binario cifrado. Distribuye copias en ubicaciones que NO contengan ninguna xpub del multisig.</p>
  </div>

  <!-- 2. Texto armored -->
  <div class="output">
    <div class="row">
      <span class="label">Texto armored</span>
      <button type="button" class="btn btn-secondary" onclick={handleCopyArmored}>{copyLabel}</button>
    </div>
    <pre class="armored" aria-label="Bloque armored del backup cifrado">{result.armored}</pre>
  </div>

  <!-- 3. QR PNG -->
  <div class="output">
    <div class="row">
      <span class="label">Código QR (PNG)</span>
      <button type="button" class="btn btn-secondary" onclick={downloadQrPng}>Descargar PNG</button>
    </div>
    <figure class="qr">
      <img
        src="data:image/png;base64,{result.qr_png_b64}"
        alt="Código QR del backup cifrado"
        width="200"
        height="200"
      />
    </figure>
  </div>
</div>

<Toast bind:visible={toastVisible} message={toastMessage} />

<style>
  .result-zone {
    margin-top: var(--space-xl);
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-card);
    padding: var(--space-lg);
    box-shadow: var(--shadow-card);
  }
  .title {
    margin: 0 0 var(--space-lg) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-heading);
  }
  .output {
    margin-bottom: var(--space-lg);
  }
  .output:last-child { margin-bottom: 0; }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md);
    margin-bottom: var(--space-sm);
    flex-wrap: wrap;
  }
  .label {
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-bold);
    color: var(--color-text-primary);
  }
  .hint {
    margin: var(--space-sm) 0 0 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    line-height: var(--line-height-label);
  }
  .armored {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    line-height: var(--line-height-mono);
    background: var(--color-surface-sunken);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-input);
    padding: var(--space-md);
    white-space: pre-wrap;
    word-break: break-all;
    overflow-x: auto;
    max-height: 200px;
    overflow-y: auto;
    margin: 0;
    color: var(--color-text-primary);
  }
  .qr {
    margin: 0;
    display: flex;
    justify-content: center;
    background: white; /* QR siempre se ve mejor sobre fondo blanco; PNG ya es blanco/negro */
    padding: var(--space-md);
    border-radius: var(--radius-input);
    border: 1px solid var(--color-border);
  }
  .btn {
    min-height: var(--touch-target);
    min-width: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    cursor: pointer;
    transition: background-color var(--transition-color), border-color var(--transition-color);
  }
  .btn-primary {
    background: var(--color-accent);
    color: var(--color-accent-fg);
    border: 0;
  }
  .btn-primary:hover { background: var(--color-accent-hover); }
  .btn-secondary {
    background: transparent;
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }
  .btn-secondary:hover { background: var(--color-surface-sunken); }
</style>
```

NO uses canvas o JS extra para el QR — el backend ya devuelve PNG base64 (D-09). Solo `<img src="data:image/png;base64,...">`.
NO descargues el QR como SVG ni lo renderices con qrcode lib — eso es para Descifrar tab (plan 02-05).
La fecha en el filename usa `toISOString().replace(/[:.]/g, '-')` para que sea filesystem-safe en Windows también.
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; npm run build 2>&amp;1 | tail -5 &amp;&amp; test -f src/components/CifrarOutputs.svelte &amp;&amp; test -f src/lib/download.js</automated>
  </verify>
  <acceptance_criteria>
    - `grep "export function triggerDownloadBytes" /workspace/descriptor-cifrado/frontend/src/lib/download.js` encuentra match
    - `grep "export function triggerDownloadBase64" /workspace/descriptor-cifrado/frontend/src/lib/download.js` encuentra match
    - `grep "export function triggerDownloadText" /workspace/descriptor-cifrado/frontend/src/lib/download.js` encuentra match
    - `grep "atob" /workspace/descriptor-cifrado/frontend/src/lib/download.js` encuentra match (base64 decode)
    - `grep "URL.createObjectURL" /workspace/descriptor-cifrado/frontend/src/lib/download.js` encuentra match
    - `grep "URL.revokeObjectURL" /workspace/descriptor-cifrado/frontend/src/lib/download.js` encuentra match (cleanup)
    - `grep "Descargar .bed" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match (UI-SPEC label literal)
    - `grep "Copiar al portapapeles" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match
    - `grep "Copiado ✓" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match
    - `grep "Descargar PNG" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match
    - `grep "data:image/png;base64" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match (QR inline)
    - `grep "1500" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match (D-34 1500ms label revert)
    - `grep "Copiado al portapapeles" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match (toast literal)
    - `grep "var(--font-mono)" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match (mono para armored)
    - `grep "var(--font-size-mono)" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` encuentra match
    - `! grep -E "#[0-9A-Fa-f]{6}" /workspace/descriptor-cifrado/frontend/src/components/CifrarOutputs.svelte` (no hex hardcoded — todo via tokens)
    - `cd /workspace/descriptor-cifrado/frontend && npm run build` exit code 0
  </acceptance_criteria>
  <done>download.js exporta 3 helpers; CifrarOutputs.svelte renderiza los 3 outputs con feedback dual toast+label; build verde.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 2: TabCifrar.svelte (form + handler) + integración con App.svelte</name>
  <files>frontend/src/components/TabCifrar.svelte, frontend/src/App.svelte</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-08 form, D-09 outputs, D-10, D-11 QrTooLarge, D-12 history opt-in, D-33 spinner inline, D-35 inline error)
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§Inputs and Textareas, §Buttons §loading state, §Copywriting Contract §Primary CTAs y §Error States y §Placeholder Copy)
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 5 — handleCifrar example)
    - frontend/src/components/CifrarOutputs.svelte (componente que recién creaste — su contrato de props)
    - frontend/src/components/InlineError.svelte (componente del plan 03 — su contrato de props)
    - frontend/src/components/Spinner.svelte (componente del plan 03)
    - frontend/src/lib/api.js (postJson + ApiError)
    - frontend/src/stores/app.svelte.js (appState.historyEnabled)
    - frontend/src/App.svelte (debes editar para montar TabCifrar; lee el placeholder actual)
  </read_first>
  <action>
1. **`frontend/src/components/TabCifrar.svelte`** — form + handleCifrar + zona de resultado. Estructura D-08, D-09:

```svelte
<script>
  import { postJson, ApiError } from '../lib/api.js';
  import { appState } from '../stores/app.svelte.js';
  import InlineError from './InlineError.svelte';
  import CifrarOutputs from './CifrarOutputs.svelte';
  import Toast from './Toast.svelte';
  import Spinner from './Spinner.svelte';

  let descriptor = $state('');
  let result = $state(null);
  let loading = $state(false);
  let errorVisible = $state(false);
  let errorMessage = $state('');
  let warningToast = $state(false);
  let warningMessage = $state('');

  const PLACEHOLDER =
    'wsh(multi(2,[fp/48h/0h/0h/2h]xpub…/<0;1>/*,[fp/48h/0h/0h/2h]xpub…/<0;1>/*))#checksum';

  async function handleCifrar() {
    if (!descriptor.trim() || loading) return;
    loading = true;
    errorVisible = false;
    errorMessage = '';
    result = null;
    try {
      const resp = await postJson('/api/encrypt', { descriptor: descriptor.trim() });
      result = resp;
      // D-12: si historyEnabled ON, persistir tras éxito.
      if (appState.historyEnabled) {
        try {
          await postJson('/api/history', { bed_b64: resp.bed_b64 });
        } catch {
          warningMessage = 'Cifrado OK, pero no se guardó en historial';
          warningToast = true;
        }
      }
    } catch (e) {
      if (e instanceof ApiError) {
        // QrTooLarge: añadir nota extra (UI-SPEC §Error States)
        if (e.code === 'QR_TOO_LARGE') {
          errorMessage = `${e.message} Usa el archivo .bed o el texto armored.`;
        } else {
          errorMessage = e.message;
        }
      } else {
        errorMessage = 'No se pudo conectar al servidor local.';
      }
      errorVisible = true;
    } finally {
      loading = false;
    }
  }

  function handleSubmit(e) {
    e.preventDefault();
    handleCifrar();
  }
</script>

<form class="form" onsubmit={handleSubmit} novalidate>
  <InlineError bind:visible={errorVisible} message={errorMessage} />

  <div class="field">
    <label for="descriptor-input" class="label">Descriptor multisig</label>
    <textarea
      id="descriptor-input"
      class="textarea"
      bind:value={descriptor}
      placeholder={PLACEHOLDER}
      spellcheck="false"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      aria-describedby="descriptor-help"
      rows="6"
    ></textarea>
    <p id="descriptor-help" class="help">
      Pega el descriptor con derivación <code>&lt;0;1&gt;/*</code>. Nada se envía a internet.
    </p>
  </div>

  <button type="submit" class="btn btn-primary" disabled={!descriptor.trim() || loading}>
    {#if loading}
      <Spinner /> <span>Cifrando…</span>
    {:else}
      <span>Cifrar</span>
    {/if}
  </button>
</form>

{#if result}
  <CifrarOutputs {result} />
{/if}

<Toast bind:visible={warningToast} message={warningMessage} />

<style>
  .form { display: flex; flex-direction: column; gap: var(--space-md); }
  .field { display: flex; flex-direction: column; gap: var(--space-sm); }
  .label {
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-label);
    color: var(--color-text-primary);
  }
  .textarea {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    line-height: var(--line-height-mono);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-input);
    background: var(--color-surface-sunken);
    color: var(--color-text-primary);
    padding: var(--space-sm) var(--space-md);
    min-height: 120px;
    resize: vertical;
    width: 100%;
    transition: border-color var(--transition-color), box-shadow var(--transition-focus);
  }
  .textarea:focus {
    outline: 0;
    border-color: var(--color-border-focus);
    box-shadow: var(--shadow-focus);
  }
  .help {
    margin: 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    line-height: var(--line-height-label);
  }
  .help code {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    background: var(--color-surface-sunken);
    padding: 1px 4px;
    border-radius: 4px;
  }
  .btn {
    min-height: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    transition: background-color var(--transition-color), opacity var(--transition-disabled);
  }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-primary {
    background: var(--color-accent);
    color: var(--color-accent-fg);
    border: 0;
  }
  .btn-primary:hover:not(:disabled) { background: var(--color-accent-hover); }
</style>
```

2. **`frontend/src/App.svelte`** — REEMPLAZAR el placeholder de la tab Cifrar montando `<TabCifrar />`:

Editar la sección actual:
```svelte
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
```

Reemplazar por:
```svelte
  <section
    role="tabpanel"
    id="panel-cifrar"
    aria-labelledby="tab-cifrar"
    class="panel"
    hidden={appState.activeTab !== 'cifrar'}
  >
    <TabCifrar />
  </section>
```

Y añadir el import en el `<script>` de App.svelte:
```svelte
import TabCifrar from './components/TabCifrar.svelte';
```

NO toques los placeholders de Descifrar e Historial (planes 02-05 y 02-06 los reemplazan).

3. Verifica build y bundle size: `cd frontend && npm run build`. El bundle JS+CSS gzipped (excluyendo woff2) debe seguir <50 KB.

4. Verifica el flujo end-to-end manualmente (smoke test):
   ```bash
   # En una terminal:
   cd /workspace/descriptor-cifrado && BED_DATA_DIR=/tmp/bed-test mkdir -p /tmp/bed-test && BED_DATA_DIR=/tmp/bed-test cargo run -p bed-server &
   # Espera ~3s al startup
   # En otra terminal:
   cd /workspace/descriptor-cifrado/frontend && npm run dev
   # Abre http://127.0.0.1:5173 — Tab Cifrar debe mostrar el form.
   ```
   Si tienes un descriptor válido a mano (uno de los tests de Phase 1), pégalo y verifica que aparece la zona de resultado con los 3 outputs.

NO añadas validación cliente del descriptor (D-08: validación inline con mensaje del backend tal cual; el backend valida via miniscript+require_multipath_0_1).
NO persistas `descriptor` ni `result` en localStorage (D-16, D-17).
NO loguees el descriptor en `console.log` ni en errores (security default).
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; npm run build 2>&amp;1 | tail -5 &amp;&amp; SIZE=$(for f in dist/assets/*.js dist/assets/*.css; do [ -f "$f" ] &amp;&amp; gzip -c "$f" | wc -c; done | awk '{s+=$1} END {print s}') &amp;&amp; echo "Bundle JS+CSS gzipped: $SIZE bytes" &amp;&amp; [ "$SIZE" -lt 51200 ]</automated>
  </verify>
  <acceptance_criteria>
    - `grep "import TabCifrar" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `grep "<TabCifrar" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `! grep "pendiente de plan 02-04" /workspace/descriptor-cifrado/frontend/src/App.svelte` (placeholder reemplazado)
    - `grep "/api/encrypt" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match
    - `grep "/api/history" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (D-12 history save)
    - `grep "appState.historyEnabled" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (gate del history call)
    - `grep "Cifrar" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (CTA literal)
    - `grep "Cifrando…" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (loading state literal)
    - `grep "Cifrado OK, pero no se guardó en historial" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (warning toast literal)
    - `grep "QR_TOO_LARGE" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (D-11 specific handling)
    - `grep "wsh(multi" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (placeholder literal del UI-SPEC)
    - `grep "<0;1>/\\*" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (help text)
    - `grep "<label for=\"descriptor-input\"" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (a11y D-37)
    - `grep "aria-describedby" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match
    - `grep "<Spinner" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (D-33)
    - `grep "<InlineError" /workspace/descriptor-cifrado/frontend/src/components/TabCifrar.svelte` encuentra match (D-35)
    - `cd /workspace/descriptor-cifrado/frontend && npm run build` exit code 0
    - Bundle JS+CSS gzipped <51200 bytes
  </acceptance_criteria>
  <done>TabCifrar montado en App.svelte; flujo Cifrar conecta a /api/encrypt; cuando historyEnabled hace fire-and-warn a /api/history; outputs renderizan los 3 formatos con feedback de copy dual; bundle <50 KB.</done>
</task>

</tasks>

<verification>
- `cd frontend && npm run build` exit code 0; bundle JS+CSS gzipped <50 KB
- TabCifrar flow: pegar descriptor → click Cifrar → spinner inline + label "Cifrando…" → respuesta 200 → 3 outputs visibles
- Si historyEnabled ON al cifrar y POST /api/history falla, toast warning sin perder el resultado
- Si descriptor inválido (422 backend) → InlineError arriba del form con mensaje literal del backend
- Copia armored: label cambia a "Copiado ✓" 1500ms + toast 3s "Copiado al portapapeles"
</verification>

<success_criteria>
- UI-02: Tab Cifrar funcional con form + 3 outputs simultáneos (D-09, D-10)
- HIST-02 (parte cliente): cuando toggle ON, fire-and-warn a POST /api/history tras éxito (D-12)
- UI-01: bundle JS+CSS gzipped <50 KB sin requests externos
- D-33 spinner inline; D-34 copy dual toast+label; D-35 inline error; D-11 QR_TOO_LARGE manejo específico
- A11y: label-for, aria-describedby, focus-visible
</success_criteria>

<output>
After completion, create `.planning/phases/02-spa-frontend-history/02-04-SUMMARY.md` describing:
- Tamaño bundle JS+CSS gzipped tras añadir TabCifrar (en bytes)
- Confirmación del flujo end-to-end smoke-tested (cifrar un descriptor real → 3 outputs)
- Lista de error codes manejados (DESCRIPTOR_PARSE, MISSING_MULTIPATH_WILDCARD, QR_TOO_LARGE, NETWORK_ERROR)
- Cualquier desviación del UI-SPEC (idealmente: ninguna)
</output>
