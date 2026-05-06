---
phase: 02-spa-frontend-history
plan: 05
type: execute
wave: 3
depends_on: [02-03]
files_modified:
  - frontend/src/App.svelte
  - frontend/src/components/TabDescifrar.svelte
  - frontend/src/components/DescifrarOutputs.svelte
  - frontend/src/components/AnimatedQrModal.svelte
  - frontend/src/lib/xpub.js
  - frontend/package.json
autonomous: true
requirements: [UI-02, DEC-04]
must_haves:
  truths:
    - "Usuario carga un .bed (drag-drop, file picker o paste armored) y una xpub (paste o file picker), pulsa 'Descifrar' y dispara POST /api/decrypt multipart"
    - "El botón 'Descifrar' está disabled hasta que ambos inputs (.bed/armored y xpub) están presentes y la xpub matchea regex `^([xyzt]pub|tpub)[A-Za-z0-9]{100,}$`"
    - "Tras éxito se muestra el descriptor recuperado en bloque <pre> mono con tres acciones: 'Copiar al portapapeles', 'Descargar .txt', 'Mostrar QR'"
    - "El descriptor recuperado vive solo en estado local del componente — desaparece al cambiar de tab o al pulsar 'Limpiar resultado'"
    - "El campo xpub se limpia automáticamente tras un descifrado exitoso (D-17)"
    - "'Mostrar QR' renderiza un QR estático si el descriptor cabe (≤500 chars) o un BBQR animado (multi-frame) si excede; ambos se cargan vía import dinámico"
    - "Errores 422 del backend (xpub incorrecta, descifrado fallido) se muestran inline arriba del form con el mensaje literal del backend"
  artifacts:
    - path: "frontend/src/components/TabDescifrar.svelte"
      provides: "Tab Descifrar: drop-zone + textarea armored + file picker .bed + textarea/file xpub + botón Descifrar + zona resultado"
    - path: "frontend/src/components/DescifrarOutputs.svelte"
      provides: "Render del descriptor recuperado con 3 acciones (copy / download .txt / mostrar QR)"
    - path: "frontend/src/components/AnimatedQrModal.svelte"
      provides: "Modal con QR estático o BBQR animado (lazy import bbqr + qrcode); botón Cerrar"
    - path: "frontend/src/lib/xpub.js"
      provides: "validateXpub(text): boolean — regex /^([xyzt]pub|tpub)[A-Za-z0-9]{100,}$/"
      exports: ["validateXpub", "XPUB_REGEX"]
  key_links:
    - from: "frontend/src/components/TabDescifrar.svelte"
      to: "frontend/src/lib/api.js"
      via: "postMultipart('/api/decrypt', formData)"
      pattern: "/api/decrypt"
    - from: "frontend/src/components/TabDescifrar.svelte"
      to: "frontend/src/lib/xpub.js"
      via: "validateXpub() para gate del botón Descifrar"
      pattern: "validateXpub"
    - from: "frontend/src/components/DescifrarOutputs.svelte"
      to: "frontend/src/lib/clipboard.js"
      via: "copyToClipboard(descriptor)"
      pattern: "copyToClipboard"
    - from: "frontend/src/components/DescifrarOutputs.svelte"
      to: "frontend/src/lib/download.js"
      via: "triggerDownloadText(descriptor, 'descriptor.txt', 'text/plain')"
      pattern: "triggerDownloadText"
    - from: "frontend/src/components/AnimatedQrModal.svelte"
      to: "bbqr (npm, lazy)"
      via: "await import('bbqr') solo cuando descriptor excede capacidad QR estática"
      pattern: "import\\('bbqr'\\)"
    - from: "frontend/src/components/AnimatedQrModal.svelte"
      to: "qrcode (npm, lazy)"
      via: "await import('qrcode') para renderizar cada frame BBQR (o el QR estático único) a canvas/img"
      pattern: "import\\('qrcode'\\)"
    - from: "frontend/src/App.svelte"
      to: "frontend/src/components/TabDescifrar.svelte"
      via: "import + montaje en tabpanel id=panel-descifrar"
      pattern: "TabDescifrar"
---

<objective>
Construir la tab "Descifrar" completa: drop-zone con drag-and-drop + textarea armored + file picker `.bed` + textarea/file xpub + botón "Descifrar" (gated por validación cliente) + zona de resultado con descriptor recuperado en bloque `<pre>` mono y tres acciones (Copiar / Descargar .txt / Mostrar QR). El "Mostrar QR" abre un modal con QR estático o BBQR animado (lazy import de `bbqr` + `qrcode` para no impactar el bundle inicial). Reemplazar el placeholder de TabDescifrar en App.svelte.

Purpose: Completar el flujo de recuperación — un holder con un `.bed` distribuido y cualquier xpub cosigner debe poder pegarlos en la app y obtener el descriptor en claro listo para importar a Sparrow/Nunchuk. Cubre UI-02 (tab Descifrar funcional) y DEC-04 (descriptor recuperado nunca se persiste; botón "Copiar al portapapeles").

Output: TabDescifrar.svelte + DescifrarOutputs.svelte + AnimatedQrModal.svelte + xpub.js helpers; package.json con `bbqr` y `qrcode` como deps; App.svelte montando el componente real.
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
@frontend/src/lib/download.js
@frontend/src/components/Modal.svelte
@frontend/src/components/InlineError.svelte
@frontend/src/components/Spinner.svelte
@frontend/src/components/Toast.svelte
@crates/server/src/routes/decrypt.rs

<interfaces>
<!-- Exports consumidos del Plan 02-03 (api/clipboard/stores) y Plan 02-04 (download). -->

From frontend/src/lib/api.js:
```javascript
export class ApiError extends Error { status; code; }
export async function postMultipart(url, formData): Promise<any>;
```

From frontend/src/lib/clipboard.js:
```javascript
export async function copyToClipboard(text: string): Promise<boolean>;
```

From frontend/src/lib/download.js (Plan 02-04):
```javascript
export function triggerDownloadText(text, filename, mime = 'text/plain'): void;
```

Backend contract (Phase 1) — `POST /api/decrypt` (multipart/form-data):
- Campos: `bed` (file: binario .bed o texto armored — backend autodetecta por bytes mágicos `-----BEGIN`) y `xpub` (texto plano o file con xpub).
- Response 200: `{ "descriptor": "wsh(...)" }`
- Response 422 (errores envelope): `{ "error": { "code": "DECRYPT_FAILED"|"DESCRIPTOR_PARSE"|"INVALID_XPUB", "message": "<castellano>" } }`

NPM packages (a añadir a frontend/package.json):
- `bbqr@^1.2.0` — encoder BBQR animado (Coinkite, Public Domain). ESM nativo, lazy import.
- `qrcode@^1.5.4` — renderer QR a canvas/dataURL. ESM, lazy import.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="false">
  <name>Task 1: lib/xpub.js + DescifrarOutputs.svelte + AnimatedQrModal.svelte (lazy QR)</name>
  <files>frontend/src/lib/xpub.js, frontend/src/components/DescifrarOutputs.svelte, frontend/src/components/AnimatedQrModal.svelte, frontend/package.json</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 6 — Descifrar autodetect; Patrón 8 — BBQR lazy import; §Bibliotecas externas npm; §Limitaciones — soporte BBQr en Nunchuk)
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§Output Action Labels — "Copiar al portapapeles", "Descargar .txt", "Mostrar QR"; §`<pre>` Result Blocks; §Modal; §Toast Messages)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-15 tres acciones; D-16 descriptor solo memoria; D-17 xpub cleared)
    - frontend/src/lib/clipboard.js (signature copyToClipboard)
    - frontend/src/lib/download.js (signature triggerDownloadText)
    - frontend/src/components/Modal.svelte (signatura del Modal compartido — props open/title/onClose; foco default en Cancel per D-36)
  </read_first>
  <action>
1. **Añadir deps a `frontend/package.json`** — `bbqr@^1.2.0` y `qrcode@^1.5.4` en `dependencies` (NO devDependencies; aunque son lazy, Vite las necesita resolver al build):

```bash
cd /workspace/descriptor-cifrado/frontend && npm install bbqr@1.2.0 qrcode@1.5.4 --save
```

Verificar que el chunk dinámico de bbqr+qrcode quede separado del bundle principal (Vite lo hace automáticamente con `import('bbqr')`).

2. **`frontend/src/lib/xpub.js`** — validador cliente. Regex aceptado por el backend (Phase 1 + RESEARCH §Patrón 6):

```javascript
// Validación cliente del xpub (formato superficial; el backend valida criptográficamente).
// Regex deriva de la spec BIP-32 + BIP-49 + BIP-84 + tpub mainnet/testnet.
export const XPUB_REGEX = /^([xyzt]pub|tpub)[A-Za-z0-9]{100,}$/;

export function validateXpub(text) {
  if (typeof text !== 'string') return false;
  const trimmed = text.trim();
  return XPUB_REGEX.test(trimmed);
}
```

3. **`frontend/src/components/AnimatedQrModal.svelte`** — modal con QR estático o BBQR animado, lazy import. El descriptor recuperado se pasa como prop `text`. Si `text.length <= 500` (umbral seguro para QR alfanumérico ECC-L) → QR único estático. Si excede → BBQR animado (frame rotation cada 600ms):

```svelte
<script>
  let { open = $bindable(false), text = '' } = $props();

  let frames = $state([]); // strings BBQR (o un solo string si es QR estático)
  let frameImages = $state([]); // dataURLs renderizadas
  let currentFrame = $state(0);
  let loading = $state(false);
  let errorMessage = $state('');
  let isAnimated = $state(false);
  let intervalId;

  // Threshold: QR alfanumérico ECC-L cabe ~700 chars; bajamos a 500 por seguridad
  // (descriptors usan caracteres mixed-case que no caben en modo alfanumérico → modo binario más restrictivo).
  const STATIC_QR_THRESHOLD = 500;

  $effect(() => {
    if (open && text) {
      loadAndRender();
    } else {
      stopAnimation();
      frames = [];
      frameImages = [];
      currentFrame = 0;
      errorMessage = '';
    }
  });

  async function loadAndRender() {
    loading = true;
    errorMessage = '';
    try {
      const QRCode = (await import('qrcode')).default;
      if (text.length <= STATIC_QR_THRESHOLD) {
        // QR único estático
        isAnimated = false;
        const dataUrl = await QRCode.toDataURL(text, {
          errorCorrectionLevel: 'L',
          margin: 2,
          width: 300,
        });
        frameImages = [dataUrl];
        currentFrame = 0;
      } else {
        // BBQR animado
        isAnimated = true;
        const { splitQRs } = await import('bbqr');
        const encoder = new TextEncoder();
        const bytes = encoder.encode(text);
        // 'U' = utf-8 string; encoding 'Z' (zlib) lo comprime; 'H' es máximo
        const result = splitQRs(bytes, 'U', { encoding: 'Z', minSplit: 1, maxSplit: 99, minVersion: 5, maxVersion: 40 });
        frames = result.parts; // array de strings BBQR
        const renderedFrames = [];
        for (const part of frames) {
          const url = await QRCode.toDataURL(part, {
            errorCorrectionLevel: 'L',
            margin: 2,
            width: 300,
          });
          renderedFrames.push(url);
        }
        frameImages = renderedFrames;
        currentFrame = 0;
        startAnimation();
      }
    } catch (e) {
      errorMessage = 'No se pudo generar el código QR. Usa "Copiar al portapapeles" o "Descargar .txt".';
    } finally {
      loading = false;
    }
  }

  function startAnimation() {
    stopAnimation();
    if (frameImages.length > 1) {
      intervalId = setInterval(() => {
        currentFrame = (currentFrame + 1) % frameImages.length;
      }, 600);
    }
  }

  function stopAnimation() {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = undefined;
    }
  }

  function handleClose() {
    stopAnimation();
    open = false;
  }
</script>

{#if open}
  <div class="backdrop" onclick={handleClose} role="presentation">
    <div
      class="panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="qr-modal-title"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="qr-modal-title" class="title">Código QR del descriptor</h2>

      {#if loading}
        <p class="hint">Generando QR…</p>
      {:else if errorMessage}
        <p class="error" role="alert">{errorMessage}</p>
      {:else if frameImages.length > 0}
        <figure class="qr">
          <img src={frameImages[currentFrame]} alt="Código QR del descriptor" width="300" height="300" />
        </figure>
        {#if isAnimated}
          <p class="hint">
            QR animado BBQR — frame {currentFrame + 1} de {frameImages.length}.
            Apunta tu cámara a la pantalla; Sparrow rota frames automáticamente.
          </p>
        {:else}
          <p class="hint">QR estático — el descriptor cabe en un solo código.</p>
        {/if}
      {/if}

      <div class="actions">
        <button type="button" class="btn btn-secondary" onclick={handleClose}>Cerrar</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(2px);
    z-index: 9000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-md);
  }
  .panel {
    background: var(--color-surface-raised);
    border-radius: var(--radius-card);
    padding: var(--space-lg);
    max-width: 400px;
    width: 100%;
    box-shadow: var(--shadow-modal);
  }
  .title {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    color: var(--color-text-primary);
  }
  .qr {
    margin: 0 0 var(--space-md) 0;
    display: flex;
    justify-content: center;
    background: white;
    padding: var(--space-md);
    border-radius: var(--radius-input);
    border: 1px solid var(--color-border);
  }
  .qr img {
    max-width: 100%;
    height: auto;
  }
  .hint {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    line-height: var(--line-height-label);
  }
  .error {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-label);
    color: var(--color-warning-text);
    background: var(--color-warning-bg);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-input);
    border-left: 4px solid var(--color-warning-border);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
  }
  .btn {
    min-height: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    cursor: pointer;
    transition: background-color var(--transition-color), border-color var(--transition-color);
  }
  .btn-secondary {
    background: transparent;
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }
  .btn-secondary:hover { background: var(--color-surface-sunken); }
</style>
```

4. **`frontend/src/components/DescifrarOutputs.svelte`** — recibe `descriptor` (string), renderiza `<pre>` y los 3 botones. Copy con feedback dual (toast + label change 1500ms, D-34). El botón "Limpiar resultado" se gestiona en TabDescifrar (Task 2), no aquí:

```svelte
<script>
  import { copyToClipboard } from '../lib/clipboard.js';
  import { triggerDownloadText } from '../lib/download.js';
  import AnimatedQrModal from './AnimatedQrModal.svelte';
  import Toast from './Toast.svelte';

  let { descriptor } = $props();

  let copyLabel = $state('Copiar al portapapeles');
  let copyResetTimer;
  let toastVisible = $state(false);
  let toastMessage = $state('');
  let qrOpen = $state(false);

  function showToast(msg) {
    toastMessage = msg;
    toastVisible = true;
  }

  async function handleCopy() {
    const ok = await copyToClipboard(descriptor);
    if (ok) {
      copyLabel = 'Copiado ✓';
      showToast('Copiado al portapapeles');
      clearTimeout(copyResetTimer);
      copyResetTimer = setTimeout(() => { copyLabel = 'Copiar al portapapeles'; }, 1500);
    } else {
      showToast('No se pudo copiar al portapapeles');
    }
  }

  function handleDownloadTxt() {
    const ts = new Date().toISOString().replace(/[:.]/g, '-');
    triggerDownloadText(descriptor, `descriptor-${ts}.txt`, 'text/plain');
  }

  function handleShowQr() {
    qrOpen = true;
  }
</script>

<div class="result-zone" aria-label="Descriptor recuperado">
  <h2 class="title">Descriptor recuperado</h2>

  <pre class="descriptor" aria-label="Descriptor multisig en claro">{descriptor}</pre>

  <div class="actions">
    <button type="button" class="btn btn-primary" onclick={handleCopy}>{copyLabel}</button>
    <button type="button" class="btn btn-secondary" onclick={handleDownloadTxt}>Descargar .txt</button>
    <button type="button" class="btn btn-secondary" onclick={handleShowQr}>Mostrar QR</button>
  </div>
</div>

<Toast bind:visible={toastVisible} message={toastMessage} />
<AnimatedQrModal bind:open={qrOpen} text={descriptor} />

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
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-heading);
    color: var(--color-text-primary);
  }
  .descriptor {
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
    margin: 0 0 var(--space-md) 0;
    color: var(--color-text-primary);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-sm);
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

NO descargues el QR como archivo (D-15: el QR se escanea de pantalla, no se descarga — consistente con el flujo de import en wallets).
NO uses `qrcode` o `bbqr` con import estático — siempre `await import(...)` lazy para no añadir al bundle inicial (~50 KB).
NO renderices el descriptor recuperado fuera del componente que lo consume — debe vivir en el `$state` local de TabDescifrar (D-16).
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; npm install bbqr@1.2.0 qrcode@1.5.4 --save 2>&amp;1 | tail -3 &amp;&amp; npm run build 2>&amp;1 | tail -8 &amp;&amp; test -f src/components/DescifrarOutputs.svelte &amp;&amp; test -f src/components/AnimatedQrModal.svelte &amp;&amp; test -f src/lib/xpub.js</automated>
  </verify>
  <acceptance_criteria>
    - `grep "\"bbqr\"" /workspace/descriptor-cifrado/frontend/package.json` encuentra match
    - `grep "\"qrcode\"" /workspace/descriptor-cifrado/frontend/package.json` encuentra match
    - `grep "export const XPUB_REGEX" /workspace/descriptor-cifrado/frontend/src/lib/xpub.js` encuentra match
    - `grep "export function validateXpub" /workspace/descriptor-cifrado/frontend/src/lib/xpub.js` encuentra match
    - `grep "\\[xyzt\\]pub|tpub" /workspace/descriptor-cifrado/frontend/src/lib/xpub.js` encuentra match (regex literal)
    - `grep "Copiar al portapapeles" /workspace/descriptor-cifrado/frontend/src/components/DescifrarOutputs.svelte` encuentra match
    - `grep "Copiado ✓" /workspace/descriptor-cifrado/frontend/src/components/DescifrarOutputs.svelte` encuentra match
    - `grep "Descargar .txt" /workspace/descriptor-cifrado/frontend/src/components/DescifrarOutputs.svelte` encuentra match
    - `grep "Mostrar QR" /workspace/descriptor-cifrado/frontend/src/components/DescifrarOutputs.svelte` encuentra match
    - `grep "1500" /workspace/descriptor-cifrado/frontend/src/components/DescifrarOutputs.svelte` encuentra match (D-34)
    - `grep "import('bbqr')" /workspace/descriptor-cifrado/frontend/src/components/AnimatedQrModal.svelte` encuentra match (lazy)
    - `grep "import('qrcode')" /workspace/descriptor-cifrado/frontend/src/components/AnimatedQrModal.svelte` encuentra match (lazy)
    - `! grep "from 'bbqr'" /workspace/descriptor-cifrado/frontend/src/components/AnimatedQrModal.svelte` (NO import estático)
    - `! grep "from 'qrcode'" /workspace/descriptor-cifrado/frontend/src/components/AnimatedQrModal.svelte` (NO import estático)
    - `grep "role=\"dialog\"" /workspace/descriptor-cifrado/frontend/src/components/AnimatedQrModal.svelte` encuentra match (D-37 a11y)
    - `grep "aria-modal=\"true\"" /workspace/descriptor-cifrado/frontend/src/components/AnimatedQrModal.svelte` encuentra match
    - `grep "splitQRs" /workspace/descriptor-cifrado/frontend/src/components/AnimatedQrModal.svelte` encuentra match (BBQR encode)
    - `grep "var(--font-mono)" /workspace/descriptor-cifrado/frontend/src/components/DescifrarOutputs.svelte` encuentra match
    - `! grep -E "#[0-9A-Fa-f]{6}" /workspace/descriptor-cifrado/frontend/src/components/DescifrarOutputs.svelte` (no hex hardcoded)
    - `cd /workspace/descriptor-cifrado/frontend && npm run build` exit code 0
    - El build de Vite genera al menos un chunk separado para bbqr+qrcode (verificar `ls dist/assets/*.js | wc -l` >= 2)
  </acceptance_criteria>
  <done>xpub.js exporta validateXpub + XPUB_REGEX; DescifrarOutputs.svelte renderiza descriptor + 3 acciones con feedback dual de copy; AnimatedQrModal.svelte hace lazy import de bbqr+qrcode y renderiza QR estático o animado según longitud; build verde con chunk dinámico separado.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 2: TabDescifrar.svelte (drop-zone + form + handler) + integración con App.svelte</name>
  <files>frontend/src/components/TabDescifrar.svelte, frontend/src/App.svelte</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-13 drop-zone + textarea + file picker; D-14 disabled gate; D-16 estado local; D-17 xpub auto-clear; D-35 inline error)
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§Drop-zone — 2px dashed border, drag-over color accent; §Inputs and Textareas — xpub min-height 72px; §Buttons; §Placeholder Copy)
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 6 — handleDescifrar con FormData; §Trampa 5 — descriptor en estado local del componente)
    - frontend/src/components/DescifrarOutputs.svelte (componente recién creado — props {descriptor})
    - frontend/src/lib/xpub.js (validateXpub)
    - frontend/src/lib/api.js (postMultipart + ApiError)
    - crates/server/src/routes/decrypt.rs (verificar nombres de campos multipart `bed` y `xpub`; verificar que acepta tanto file como text plain en ambos)
    - frontend/src/App.svelte (placeholder a reemplazar)
  </read_first>
  <action>
1. **`frontend/src/components/TabDescifrar.svelte`** — drop-zone para .bed binario + textarea armored (con autodetect del backend) + file picker .bed + textarea xpub + file picker xpub + botón Descifrar gated. Estado descriptor SOLO local del componente (D-16):

```svelte
<script>
  import { postMultipart, ApiError } from '../lib/api.js';
  import { validateXpub } from '../lib/xpub.js';
  import InlineError from './InlineError.svelte';
  import DescifrarOutputs from './DescifrarOutputs.svelte';
  import Spinner from './Spinner.svelte';

  // Estado LOCAL — al desmontar (cambio de tab) Svelte lo descarta automáticamente (D-16, D-17).
  let bedFile = $state(null);          // File object si vino de drop o file picker .bed
  let bedFilename = $state('');        // nombre visible del archivo cargado
  let armoredText = $state('');        // textarea armored
  let xpubText = $state('');
  let descriptor = $state(null);       // resultado descifrado — vive solo aquí
  let loading = $state(false);
  let errorVisible = $state(false);
  let errorMessage = $state('');
  let dragOver = $state(false);
  let bedFileInput;
  let xpubFileInput;

  // El botón Descifrar está disabled hasta que tenemos:
  // (a) un .bed (file binario) O un texto armored no vacío
  // (b) una xpub que matchea el regex
  let bedReady = $derived(bedFile !== null || armoredText.trim().length > 0);
  let xpubReady = $derived(validateXpub(xpubText));
  let canSubmit = $derived(bedReady && xpubReady && !loading);

  function clearAll() {
    bedFile = null;
    bedFilename = '';
    armoredText = '';
    xpubText = '';
    descriptor = null;
    errorVisible = false;
    errorMessage = '';
  }

  function handleClearResult() {
    descriptor = null;
  }

  function handleDragOver(e) {
    e.preventDefault();
    dragOver = true;
  }
  function handleDragLeave() {
    dragOver = false;
  }
  function handleDrop(e) {
    e.preventDefault();
    dragOver = false;
    const file = e.dataTransfer?.files?.[0];
    if (file) {
      acceptBedFile(file);
    }
  }
  function handleBedFilePick(e) {
    const file = e.target?.files?.[0];
    if (file) acceptBedFile(file);
  }
  function acceptBedFile(file) {
    bedFile = file;
    bedFilename = file.name;
    // Si el usuario cargó un binario, vaciar el textarea armored para evitar confusión.
    armoredText = '';
  }
  function clearBedFile() {
    bedFile = null;
    bedFilename = '';
    if (bedFileInput) bedFileInput.value = '';
  }
  function handleArmoredInput() {
    // Si el usuario escribe armored, descartar el file binario para evitar enviar ambos.
    if (armoredText.trim().length > 0 && bedFile) {
      clearBedFile();
    }
  }
  async function handleXpubFilePick(e) {
    const file = e.target?.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      xpubText = text.trim();
    } catch {
      errorMessage = 'No se pudo leer el archivo de xpub.';
      errorVisible = true;
    }
    if (xpubFileInput) xpubFileInput.value = '';
  }

  async function handleDescifrar() {
    if (!canSubmit) return;
    loading = true;
    errorVisible = false;
    errorMessage = '';
    descriptor = null;
    try {
      const formData = new FormData();
      if (bedFile) {
        formData.append('bed', bedFile, bedFile.name);
      } else {
        // Enviar armored como Blob text (el backend autodetecta por bytes mágicos).
        const armoredBlob = new Blob([armoredText.trim()], { type: 'text/plain' });
        formData.append('bed', armoredBlob, 'armored.txt');
      }
      formData.append('xpub', xpubText.trim());

      const resp = await postMultipart('/api/decrypt', formData);
      descriptor = resp.descriptor;
      // D-17: limpiar xpub tras descifrado exitoso (security default).
      xpubText = '';
    } catch (e) {
      if (e instanceof ApiError) {
        errorMessage = e.message;
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
    handleDescifrar();
  }
</script>

<form class="form" onsubmit={handleSubmit} novalidate>
  <InlineError bind:visible={errorVisible} message={errorMessage} />

  <!-- Sección 1: .bed o armored -->
  <fieldset class="field">
    <legend class="legend">Backup cifrado (`.bed` o armored)</legend>

    <div
      class="dropzone"
      class:dragover={dragOver}
      role="button"
      tabindex="0"
      aria-label="Soltar archivo .bed aquí o pulsa para seleccionar"
      ondragover={handleDragOver}
      ondragleave={handleDragLeave}
      ondrop={handleDrop}
      onclick={() => bedFileInput?.click()}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); bedFileInput?.click(); } }}
    >
      {#if bedFile}
        <p class="dropzone-text">
          Archivo cargado: <strong class="filename">{bedFilename}</strong>
        </p>
        <button type="button" class="btn btn-ghost" onclick={(e) => { e.stopPropagation(); clearBedFile(); }}>
          Cambiar archivo
        </button>
      {:else}
        <p class="dropzone-text">Arrastra el archivo `.bed` aquí o pulsa para seleccionar</p>
      {/if}
      <input
        bind:this={bedFileInput}
        type="file"
        accept=".bed,application/octet-stream"
        hidden
        onchange={handleBedFilePick}
      />
    </div>

    <p class="separator">— o pega el texto armored —</p>

    <label for="armored-input" class="label visually-hidden">Texto armored</label>
    <textarea
      id="armored-input"
      class="textarea"
      bind:value={armoredText}
      oninput={handleArmoredInput}
      placeholder="-----BEGIN BITCOIN ENCRYPTED BACKUP-----"
      spellcheck="false"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      rows="4"
      disabled={bedFile !== null}
    ></textarea>
  </fieldset>

  <!-- Sección 2: xpub -->
  <fieldset class="field">
    <legend class="legend">xpub cosigner</legend>

    <label for="xpub-input" class="label visually-hidden">xpub cosigner</label>
    <textarea
      id="xpub-input"
      class="textarea xpub"
      bind:value={xpubText}
      placeholder="xpub6... o tpub…"
      spellcheck="false"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      rows="3"
      aria-describedby="xpub-help"
    ></textarea>
    <p id="xpub-help" class="help">
      Cualquier xpub cosigner del multisig sirve. La xpub se borra automáticamente tras descifrar.
    </p>

    <button type="button" class="btn btn-ghost" onclick={() => xpubFileInput?.click()}>
      Subir archivo con xpub
    </button>
    <input
      bind:this={xpubFileInput}
      type="file"
      accept=".txt,text/plain"
      hidden
      onchange={handleXpubFilePick}
    />
  </fieldset>

  <button type="submit" class="btn btn-primary" disabled={!canSubmit}>
    {#if loading}
      <Spinner /> <span>Descifrando…</span>
    {:else}
      <span>Descifrar</span>
    {/if}
  </button>
</form>

{#if descriptor}
  <div class="result-wrapper">
    <DescifrarOutputs {descriptor} />
    <button type="button" class="btn btn-ghost clear-btn" onclick={handleClearResult}>
      Limpiar resultado
    </button>
  </div>
{/if}

<style>
  .form { display: flex; flex-direction: column; gap: var(--space-lg); }
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    border: 0;
    padding: 0;
    margin: 0;
  }
  .legend {
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-bold);
    color: var(--color-text-primary);
    margin-bottom: var(--space-sm);
    padding: 0;
  }
  .dropzone {
    border: 2px dashed var(--color-border);
    border-radius: var(--radius-card);
    background: var(--color-surface-sunken);
    padding: var(--space-xl) var(--space-md);
    text-align: center;
    cursor: pointer;
    transition: border-color var(--transition-color), background-color var(--transition-color);
  }
  .dropzone:hover, .dropzone:focus-visible {
    border-color: var(--color-border-focus);
    outline: 0;
  }
  .dropzone.dragover {
    border-color: var(--color-accent);
    background: var(--color-surface-raised);
  }
  .dropzone-text {
    margin: 0 0 var(--space-sm) 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
  }
  .filename {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    color: var(--color-text-primary);
  }
  .separator {
    text-align: center;
    margin: var(--space-sm) 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
  }
  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0,0,0,0);
    border: 0;
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
    min-height: 100px;
    resize: vertical;
    width: 100%;
    transition: border-color var(--transition-color), box-shadow var(--transition-focus);
  }
  .textarea.xpub { min-height: 72px; }
  .textarea:focus {
    outline: 0;
    border-color: var(--color-border-focus);
    box-shadow: var(--shadow-focus);
  }
  .textarea:disabled { opacity: 0.5; }
  .help {
    margin: 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    line-height: var(--line-height-label);
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
  .btn-ghost {
    background: transparent;
    color: var(--color-text-secondary);
    border: 0;
    align-self: flex-start;
  }
  .btn-ghost:hover { background: var(--color-surface-sunken); color: var(--color-text-primary); }
  .clear-btn { margin-top: var(--space-md); }
  .result-wrapper { display: flex; flex-direction: column; }
</style>
```

2. **`frontend/src/App.svelte`** — REEMPLAZAR el placeholder de la tab Descifrar montando `<TabDescifrar />`:

Buscar la sección actual:
```svelte
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
```

Reemplazar por:
```svelte
  <section
    role="tabpanel"
    id="panel-descifrar"
    aria-labelledby="tab-descifrar"
    class="panel"
    hidden={appState.activeTab !== 'descifrar'}
  >
    <TabDescifrar />
  </section>
```

Y añadir el import en el `<script>` de App.svelte:
```svelte
import TabDescifrar from './components/TabDescifrar.svelte';
```

NO toques el placeholder de la tab Historial (Plan 02-06 lo reemplaza).
NO uses `setActiveTab` ni stores globales para `descriptor` — debe vivir en `$state` LOCAL del componente para que se descarte al desmontar (D-16, comprobado en RESEARCH §Trampa 5).

3. Verifica que el build de Vite sigue verde y que el bundle inicial gzipped (excluyendo el chunk dinámico de bbqr+qrcode) sigue por debajo del presupuesto.

4. Smoke test manual:
   ```bash
   # Terminal 1
   cd /workspace/descriptor-cifrado && BED_DATA_DIR=/tmp/bed-test mkdir -p /tmp/bed-test && BED_DATA_DIR=/tmp/bed-test cargo run -p bed-server &
   # Terminal 2
   cd /workspace/descriptor-cifrado/frontend && npm run dev
   # Abrir http://127.0.0.1:5173, ir a Descifrar.
   # Test: cifra un descriptor en Tab Cifrar, descarga el .bed, vuelve a Descifrar, cárgalo + xpub conocida del multisig de prueba.
   # Verificar: descriptor recuperado aparece, xpub se vacía sola, "Mostrar QR" abre modal con QR animado o estático.
   # Cambiar a Tab Cifrar y volver a Descifrar — el descriptor recuperado debe haber desaparecido (D-16).
   ```

NO loguees el descriptor recuperado, ni la xpub, ni en `console.log` ni en mensajes de error (security default; el backend ya tiene SEC-01 con TraceLayer skip_all).
NO almacenes `descriptor` o `xpubText` en localStorage ni en sessionStorage.
NO añadas `setActiveTab` que preserve el descriptor entre tabs.
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; npm run build 2>&amp;1 | tail -8 &amp;&amp; INITIAL=$(for f in dist/assets/index-*.js dist/assets/index-*.css; do [ -f "$f" ] &amp;&amp; gzip -c "$f" | wc -c; done | awk '{s+=$1} END {print s}') &amp;&amp; echo "Bundle inicial JS+CSS gzipped: $INITIAL bytes" &amp;&amp; [ "$INITIAL" -lt 51200 ]</automated>
  </verify>
  <acceptance_criteria>
    - `grep "import TabDescifrar" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `grep "<TabDescifrar" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `! grep "pendiente de plan 02-05" /workspace/descriptor-cifrado/frontend/src/App.svelte` (placeholder reemplazado)
    - `grep "/api/decrypt" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match
    - `grep "FormData" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (multipart contract)
    - `grep "validateXpub" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (gate del botón)
    - `grep "Descifrar" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (CTA)
    - `grep "Descifrando…" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (loading literal)
    - `grep "Limpiar resultado" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (D-16)
    - `grep "xpubText = ''" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (D-17 auto-clear)
    - `grep "ondrop" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (drag-and-drop)
    - `grep "ondragover" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match
    - `grep "BEGIN BITCOIN ENCRYPTED BACKUP" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (placeholder armored literal)
    - `grep "xpub6" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (placeholder xpub literal)
    - `grep "<Spinner" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (D-33)
    - `grep "<InlineError" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` encuentra match (D-35)
    - `! grep "localStorage" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` (D-16/D-17 — descriptor/xpub nunca persisten)
    - `! grep "sessionStorage" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte`
    - `! grep "console.log" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` (security default)
    - `! grep -E "#[0-9A-Fa-f]{6}" /workspace/descriptor-cifrado/frontend/src/components/TabDescifrar.svelte` (no hex hardcoded)
    - `cd /workspace/descriptor-cifrado/frontend && npm run build` exit code 0
    - Bundle inicial JS+CSS gzipped (sin contar el chunk dinámico de bbqr/qrcode) <51200 bytes
  </acceptance_criteria>
  <done>TabDescifrar montado en App.svelte; flujo Descifrar funciona con drop-zone + textarea armored + file picker + xpub textarea/file; botón gated por validateXpub + bedReady; descriptor recuperado renderizado vía DescifrarOutputs y vive solo en estado local; xpub se limpia tras éxito; bundle inicial respeta presupuesto.</done>
</task>

</tasks>

<verification>
- `cd frontend && npm run build` exit code 0; bundle inicial JS+CSS gzipped <50 KB; chunk dinámico bbqr+qrcode separado
- TabDescifrar flow: drop o pegar armored + xpub válida → click Descifrar → spinner inline + "Descifrando…" → respuesta 200 → descriptor en `<pre>` con 3 acciones
- xpub se vacía automáticamente tras descifrado exitoso (D-17)
- Cambiar a otra tab y volver a Descifrar — descriptor recuperado desaparecido (D-16)
- "Mostrar QR" abre modal con QR estático o BBQR animado según longitud; bbqr y qrcode SOLO se cargan cuando se pulsa el botón
- Errores 422 (xpub incorrecta, descifrado fallido) → InlineError arriba del form con mensaje literal del backend
- Drop, file picker .bed y textarea armored son mutuamente excluyentes (cargar uno limpia el otro)
</verification>

<success_criteria>
- UI-02 (parte Descifrar): tab funcional con drop-zone + 3 acciones de output (D-13, D-15)
- DEC-04: descriptor recuperado nunca persiste — vive solo en `$state` local del componente; al desmontar Svelte lo descarta (D-16); botón "Limpiar resultado" disponible
- D-14 botón gated por validateXpub + bedReady
- D-17 xpub auto-cleared tras éxito
- D-33 spinner inline; D-34 copy dual toast+label; D-35 inline error
- BBQR + qrcode lazy-imported (chunk dinámico) — bundle inicial respeta presupuesto <50 KB
- A11y: drop-zone como button con keyboard (Enter/Space), label visualmente oculto pero asociado, aria-describedby en xpub
</success_criteria>

<output>
After completion, create `.planning/phases/02-spa-frontend-history/02-05-SUMMARY.md` describing:
- Tamaño bundle inicial JS+CSS gzipped tras añadir TabDescifrar (en bytes)
- Tamaño del chunk dinámico bbqr+qrcode (en bytes, gzipped)
- Confirmación del flujo end-to-end smoke-tested (descifrar un .bed real → descriptor recuperado → Mostrar QR funciona)
- Versiones exactas instaladas de bbqr y qrcode (de package-lock.json)
- Cualquier desviación del UI-SPEC (idealmente: ninguna)
- Confirmación de que cambiar de tab limpia el descriptor recuperado (D-16 verificado)
</output>
</content>
</invoke>