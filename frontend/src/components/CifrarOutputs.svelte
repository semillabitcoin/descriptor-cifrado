<script>
  import { copyToClipboard } from '../lib/clipboard.js';
  import { triggerDownloadBase64 } from '../lib/download.js';
  import { sanitizeLabelForFilename } from '../lib/labelSanitize.js';
  import Toast from './Toast.svelte';

  let { result, label = '' } = $props(); // { bed_b64, armored, qr_png_b64? }, label requerido

  let copyLabel = $state('Copiar al portapapeles');
  let copyResetTimer;
  let toastVisible = $state(false);
  let toastMessage = $state('');

  function showToast(msg) {
    toastMessage = msg;
    toastVisible = true;
  }

  function safeStem() {
    return sanitizeLabelForFilename(label) || 'backup';
  }

  function downloadBed() {
    triggerDownloadBase64(result.bed_b64, `${safeStem()}.bed`, 'application/octet-stream');
  }

  function downloadQrPng() {
    triggerDownloadBase64(result.qr_png_b64, `${safeStem()}.png`, 'image/png');
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
    <p class="hint">Binario cifrado. Distribuye copias en ubicaciones que NO contengan ninguna xpub que pertenezca al descriptor.</p>
  </div>

  <!-- 2. Texto armored -->
  <div class="output">
    <div class="row">
      <span class="label">Texto armored</span>
      <button type="button" class="btn btn-secondary" onclick={handleCopyArmored}>{copyLabel}</button>
    </div>
    <pre class="armored" aria-label="Bloque armored del backup cifrado">{result.armored}</pre>
  </div>

  <!-- 3. QR PNG (omitido si payload excede capacidad QR) -->
  {#if result.qr_png_b64}
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
  {:else}
    <div class="output">
      <p class="hint">QR no disponible: el payload excede la capacidad del código QR (máx. 2900 B). Usa el archivo .bed o el texto armored.</p>
    </div>
  {/if}
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
    background: var(--color-qr-bg, white);
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
