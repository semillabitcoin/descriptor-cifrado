<script>
  import { getJson, ApiError } from '../lib/api.js';
  import { copyToClipboard } from '../lib/clipboard.js';
  import { triggerDownloadBase64 } from '../lib/download.js';
  import { sanitizeLabelForFilename } from '../lib/labelSanitize.js';
  import Spinner from './Spinner.svelte';
  import Toast from './Toast.svelte';

  let { open = $bindable(false), entryId = '', filename = '', label = '' } = $props();

  let loading = $state(false);
  let errorMessage = $state('');
  let result = $state(null); // { bed_b64, armored, qr_png_b64? }
  let copyLabel = $state('Copiar al portapapeles');
  let copyResetTimer;
  let toastVisible = $state(false);
  let toastMessage = $state('');

  $effect(() => {
    if (open && entryId && !result) {
      void loadDetail();
    }
    if (!open) {
      result = null;
      errorMessage = '';
      copyLabel = 'Copiar al portapapeles';
      clearTimeout(copyResetTimer);
    }
  });

  async function loadDetail() {
    loading = true;
    errorMessage = '';
    try {
      result = await getJson(`/api/history/${entryId}`);
    } catch (e) {
      if (e instanceof ApiError) {
        if (e.status === 404) {
          errorMessage = 'Esta entrada ya no existe en el historial.';
        } else {
          errorMessage = e.message;
        }
      } else {
        errorMessage = 'No se pudo conectar al servidor local.';
      }
    } finally {
      loading = false;
    }
  }

  function showToast(msg) {
    toastMessage = msg;
    toastVisible = true;
  }

  function downloadStem() {
    const fromLabel = sanitizeLabelForFilename(label);
    if (fromLabel) return fromLabel;
    return (filename || `${entryId}.bed`).replace(/\.bed$/i, '');
  }

  function downloadBed() {
    if (!result) return;
    triggerDownloadBase64(result.bed_b64, `${downloadStem()}.bed`, 'application/octet-stream');
  }

  function downloadQrPng() {
    if (!result) return;
    triggerDownloadBase64(result.qr_png_b64, `${downloadStem()}.png`, 'image/png');
  }

  async function handleCopyArmored() {
    if (!result) return;
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

  function handleClose() { open = false; }

  function handleBackdropKeydown(e) {
    if (e.key === 'Escape') { e.preventDefault(); handleClose(); }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={handleClose} onkeydown={handleBackdropKeydown} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="detail-title"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="detail-title" class="title">Backup cifrado</h2>
      {#if label}
        <p class="subtitle-label">{label}</p>
        <p class="subtitle">{filename}</p>
      {:else}
        <p class="subtitle">{filename}</p>
      {/if}

      {#if loading}
        <p class="hint"><Spinner /> Cargando…</p>
      {:else if errorMessage}
        <p class="error" role="alert">{errorMessage}</p>
      {:else if result}
        <div class="output">
          <div class="row">
            <span class="label">Archivo .bed</span>
            <button type="button" class="btn btn-primary" onclick={downloadBed}>Descargar .bed</button>
          </div>
        </div>

        <div class="output">
          <div class="row">
            <span class="label">Texto armored</span>
            <button type="button" class="btn btn-secondary" onclick={handleCopyArmored}>{copyLabel}</button>
          </div>
          <pre class="armored">{result.armored}</pre>
        </div>

        {#if result.qr_png_b64}
          <div class="output">
            <div class="row">
              <span class="label">Código QR (PNG)</span>
              <button type="button" class="btn btn-secondary" onclick={downloadQrPng}>Descargar PNG</button>
            </div>
            <figure class="qr">
              <img src="data:image/png;base64,{result.qr_png_b64}" alt="Código QR del backup cifrado" width="180" height="180" />
            </figure>
          </div>
        {:else}
          <div class="output">
            <p class="hint">QR no disponible: payload excede capacidad QR (máx. 2900 B). Usa .bed o armored.</p>
          </div>
        {/if}
      {/if}

      <div class="actions">
        <button type="button" class="btn btn-secondary" onclick={handleClose}>Cerrar</button>
      </div>
    </div>
  </div>
{/if}

<Toast bind:visible={toastVisible} message={toastMessage} />

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.4); backdrop-filter: blur(2px); z-index: 9000; display: flex; align-items: center; justify-content: center; padding: var(--space-md); }
  .panel { background: var(--color-surface-raised); border-radius: var(--radius-card); padding: var(--space-lg); max-width: 480px; width: 100%; max-height: 90vh; overflow-y: auto; box-shadow: var(--shadow-modal); }
  .title { margin: 0 0 var(--space-xs) 0; font-size: var(--font-size-heading); font-weight: var(--font-weight-bold); color: var(--color-text-primary); }
  .subtitle-label { margin: 0 0 var(--space-xs) 0; font-size: var(--font-size-body); font-weight: var(--font-weight-bold); color: var(--color-text-primary); word-break: break-word; }
  .subtitle { margin: 0 0 var(--space-md) 0; font-family: var(--font-mono); font-size: var(--font-size-mono); color: var(--color-text-secondary); }
  .output { margin-bottom: var(--space-md); }
  .row { display: flex; justify-content: space-between; align-items: center; gap: var(--space-md); margin-bottom: var(--space-sm); flex-wrap: wrap; }
  .label { font-size: var(--font-size-label); font-weight: var(--font-weight-bold); color: var(--color-text-primary); }
  .armored { font-family: var(--font-mono); font-size: var(--font-size-mono); line-height: var(--line-height-mono); background: var(--color-surface-sunken); border: 1px solid var(--color-border); border-radius: var(--radius-input); padding: var(--space-md); white-space: pre-wrap; word-break: break-all; max-height: 160px; overflow-y: auto; margin: 0; }
  .qr { margin: 0; display: flex; justify-content: center; background: white; padding: var(--space-md); border-radius: var(--radius-input); border: 1px solid var(--color-border); }
  .hint { font-size: var(--font-size-label); color: var(--color-text-secondary); display: flex; align-items: center; gap: var(--space-sm); }
  .error { margin: 0 0 var(--space-md) 0; font-size: var(--font-size-label); color: var(--color-warning-text); background: var(--color-warning-bg); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-input); border-left: 4px solid var(--color-warning-border); }
  .actions { display: flex; justify-content: flex-end; gap: var(--space-sm); margin-top: var(--space-md); }
  .btn { min-height: var(--touch-target); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-button); font-size: var(--font-size-label); cursor: pointer; transition: background-color var(--transition-color); }
  .btn-primary { background: var(--color-accent); color: var(--color-accent-fg); border: 0; }
  .btn-primary:hover { background: var(--color-accent-hover); }
  .btn-secondary { background: transparent; color: var(--color-text-primary); border: 1px solid var(--color-border); }
  .btn-secondary:hover { background: var(--color-surface-sunken); }
</style>
